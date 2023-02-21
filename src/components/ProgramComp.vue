<template>
    <div class="wrap">
      <div v-drag="{ handle: '#debugDragHandle' }">
        <DebugPanel
          :isActive="isActive"
          @next="next"
          @continue="continue_"
          @restart="restart"
          @stop="stop"
        />
      </div>
      <button class="add-item" @click="addItem" title="add panel">+</button>
    </div>
    <div class="content">
      <grid-layout
        v-model:layout="layout"
        :col-num="colNum"
        :row-height="30"
        :is-draggable="true"
        :is-resizable="true"
        :vertical-compact="true"
        :use-css-transforms="true"
      >
        <grid-item
          v-for="item in layout"
          :key="item.i"
          :x="item.x"
          :y="item.y"
          :w="item.w"
          :h="item.h"
          :i="item.i"
          :is-resizable="item.isResizable"
        >    
        <div class="dis-view">

        <div v-if="item.name !== 'editor'" class="panel-header">
          <p class="title">{{ item.name }}</p>
          <div class="delete-container" title="remove">
          <span class="delete" @click="removeItem(item.i)">x</span>  
          </div>
        </div> 
          <component
            :node="tree"
            :line="line"
            :file="file"
            :focus="focus"
            :disData="disData"
            :breakpoints="breakpoints"
            :breakpointsEditor="breakpointsEditor"
            :breakpointsEditorRemove="breakpointsEditorRemove"
            :lldbOutput="lldbOutput"
            @change-file="changeFile"
            @toggle-folder="toggleFolder"
            @breakpoint="breakpoint"
            @deleteBreakpoint="deleteBreakpoint"
            @executeCommand="executeLLDBCommand"
            @changeName="changeName($event, item.i)"
            v-if="item.isComponent"
            :is="item.c"
          ></component>
          <div v-else v-html="item.c"></div>
          </div>
        </grid-item>
      </grid-layout>
    </div>
</template>

<script>
import AddBtn from "./AddBtn.vue";
import EditorComponent from "./Editor.vue";
import DebugPanel from "./DebugPanel.vue";
import TreeNode from "./TreeNode.vue";
import DisassemblyComp from "./DisassemblyComp.vue";
import BreakpointComp from "./BreakpointComp.vue";
import LLDBComp from "./LLDBComp.vue";
import NewComp from "./NewComp.vue";
import lldbModule from "../lldb";

const startLayout = [
  {
    x: 1,
    y: 1,
    w: 5,
    h: 20,
    i: "0",
    name: "editor",
    c: "EditorComponent",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 0,
    y: 0,
    w: 1,
    h: 10,
    i: "1",
    name: "files",
    c: "TreeNode",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 6,
    y: 0,
    w: 3,
    h: 8,
    i: "2",
    name: "disassembly",
    c: "DisassemblyComp",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 6,
    y: 0,
    w: 2,
    h: 8,
    i: "3",
    name: "breakpoints",
    c: "BreakpointComp",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 6,
    y: 3,
    w: 3,
    h: 8,
    i: "4",
    name: "lldb command",
    c: "LLDBComp",
    isResizable: true,
    isComponent: true,
  },
];

// const treeData = {
// {
//         name: "Code",
//         children: [
//           {
//             name: "child folder 1",
//             children: [

//               { name: "hello" },
//               { name: "wat" },

//             ]
//           },
//           }
//       };

export default {
  name: "App",
  components: {
    AddBtn,
    EditorComponent,
    DebugPanel,
    TreeNode,
    DisassemblyComp,
    BreakpointComp,
    LLDBComp,
    NewComp,
  },
  data() {
    return {
      LLDB: null,
      layout: startLayout,
      index: 0,
      colNum: 12,
      file: {},
      line: 1,
      tree: {},
      prev_node: null,
      seqId: 0,
      focus: false,
      isActive: true,
      disData: [],
      breakpoints: {},
      breakpointsEditor: [],
      breakpointsEditorRemove: null,
      lldbOutput: '',
      tx_hash: '',
      inst_nr: 0,
    };
  },
  async created() {
      // Fetch lldb.wasm
      // new lldbModule({
      //   locateFile(path) {
      //     if (path.endsWith(`.wasm`)) {
      //       return "http://localhost:8003/lldb.wasm";
      //     }
      //     return path;
      //   },
      // }).then((myModule) => {
      //   this.LLDB = myModule;
      //   console.log("LOADED WASM");
      // });
    this.LLDB = await this.fetchLLDB();
    // vue-grid-layout
    this.index = this.layout.length;
    this.tx_hash = this.$route.query.tx_hash;
    this.inst_nr = this.$route.query.inst_nr;
    console.log( this.$route.query)
    await this.getTree(this.$route.query.program_id);
    await this.loadElf(this.$route.query.program_id);
    // CPI
    let res = await this.LLDB.ccall("execute_command", "number", ["string"], ["b solana_program::program::invoke_signed_unchecked"]);
    await this.LLDB._free(res);
    await this.connect();
    // await this.updateEditor();
    console.log("mounted");
    },
  methods: {
    async fetchLLDB() {
      return new lldbModule({
        locateFile(path) {
          if (path.endsWith(`.wasm`)) {
            return "http://localhost:8003/lldb.wasm";
          }
          return path;
        },
      })
    },
    // Components
    removeItem(val) {
      const index = this.layout.findIndex(item => item.i === val);
      this.layout.splice(index, 1);    
    },
    addItem() {
            this.layout.push({
                x: 2,
                y: 1,
                w: 3,
                h: 10,
                i: this.index,
                name: "",
                c: "NewComp",
                isResizable: true,
                isComponent: true,
            });
            this.index++;
      },
    changeName(name, val) {
      console.log("change", name, val);
            const index = this.layout.findIndex(item => item.i === val);
            this.layout[index].name = name;
        },
    // LLDB commands
    async loadElf(program_id) {
      var data = await fetch("http://localhost:8003/elfs/" + program_id + ".so");
      data = await data["arrayBuffer"]();
      console.log(data);
      this.LLDB.FS.writeFile("program.so", new Uint8Array(data));
      let res = this.LLDB.ccall("create_target", null, ["string"], ["program.so"]);
      this.LLDB._free(res);
    },
    async connect() {
      this.LLDB['websocket']['url'] = "ws://localhost:9007/?token=" + this.tx_hash + "_" + this.inst_nr;
      const res = await this.LLDB.ccall('execute_command', 'number', ['string'], ['gdb-remote 9007']);
      this.LLDB._free(res);
    },
    async executeLLDBCommand(command) {
      console.log("executeLLDBCommand", command);
      let resPtr = await this.LLDB.ccall("execute_command", "number", ["string"], [command]);
      this.lldbOutput = await this.LLDB.UTF8ToString(resPtr);
      this.LLDB._free(resPtr);
    },
    // Debug Panel
    async next() {
      this.isActive = false;
      console.log("next");
      // await this.LLDB.ccall("request_next_with_cpi", null, [], []);
      const res = await this.LLDB.ccall("execute_command", "number", ["string"], ['next']);
      this.LLDB._free(res);
      await this.updateEditor();
      this.isActive = true;
    },
    async continue_() {
      this.isActive = false;
      console.log("continue");
      await this.LLDB.ccall("request_continue", null, [], []);
      this.updateEditor();
      this.isActive = true;
    },
    async restart() {
      console.log("restart");
      await this.LLDB.ccall("request_next", null, [], []);
      this.updateEditor();
    },
    async stop() {
      console.log("stop");
      await this.LLDB.ccall("request_next", null, [], []);
      this.updateEditor();
    },
    // Update
    async updateEditor() {
      console.log("update editor")
      var request = { 
        "type": "request",
        "seq": this.seqId,
        "command": "stackTrace",
        "arguments": { "threadId": 0, "startFrame": 0, "levels": 10 }};
        console.log("request", request)
      let rxPtr = await this.LLDB._malloc(request.length + 1);
      await this.LLDB.stringToUTF8(request, rxPtr, request.length + 1);
      const txPtr = await this.LLDB.ccall(
        "request_stackTrace",
        "number",
        ["number"],
        [rxPtr],
      );
      const responseStr = await this.LLDB.UTF8ToString(txPtr);
      let responseJSON = JSON.parse(responseStr);
      console.log("response", responseJSON);
      this.seqId++;
      console.log("PATH:", responseJSON.body.stackFrames[0].source.path)
      let file = this.sanitizeFileName(responseJSON.body.stackFrames[0].source.path);
      if (this.file.name !== file) {
        this.file.name = file;
        this.file.line = responseJSON.body.stackFrames[0].line;
        this.load_file(file);
        this.file = {"name": file, "line": responseJSON.body.stackFrames[0].line}
      } else {
      this.line = responseJSON.body.stackFrames[0].line;
      }
      await this.disassemble();
    },

    async disassemble() {
      let res = await this.LLDB.ccall('execute_command', 'string', ['string'], ['disassemble -p -b -c 7']);
      console.log("dis", res.split("\n"));
      res = res.split("\n").slice(1);
      res[0] = res[0].slice(3) // Remove leading arrow
      let data = []
      for (let line of res) {
        try {
        let addr = line.split(' <')[0];
        let inst = line.split(': ')[1].slice(0, 23);
        let mnem = line.split(inst)[1];
        data.push({'addr': addr, 'inst': inst.toUpperCase(), 'mnem': mnem})
      } catch (e) {
          //console.log("error", e)
        }
      }
      this.disData = data;
    },

    sanitizeFileName(fileName) {
      if (fileName.includes('solana-program-1.10.33')) {
        fileName = 'code/sdk/program/' + fileName.split('solana-program-1.10.33/')[1]
      }
      else if (fileName.includes('rust-own')) {
        fileName = 'code/rust-solana-1.59.0/' + fileName.split('rust-own/rust/')[1]
      }
      else if (fileName.includes('program-rust')) {
        fileName = 'code/' + fileName.split('program-rust/')[1]
      }
      console.log("sanitized:", fileName)
      return fileName;
    },

    // Setup
    async getTree(program_id) {
      const res = await fetch("http://localhost:8003/trees/"+program_id+".json");
      const finalRes = await res.json();
      this.tree = finalRes;
      console.log("APP getTree", this.tree);
      this.load_file("code/src/lib.rs");
      this.file = {"name": "code/src/lib.rs", line: 1};
    },
    // Editor
    breakpoint(line) {
      console.log("breakpoint", line);
      let breakpoints = [];
      let exists = false;
      if (this.breakpoints[this.file.name] === undefined) {
        console.log("NEWW")
        this.breakpoints[this.file.name] = [];
      }
      else { 
      for (let l of this.breakpoints[this.file.name]) {
        if (l === line) {
          console.log("breakpoint already exists -> removing");
          exists = true;
          continue;
        }
        breakpoints.push(l);
      }
    }
      if (!exists) {
        breakpoints.push(line);
      }
      console.log("breakpoints new array", breakpoints)
      console.log("file name", this.file.name)
      this.breakpoints[this.file.name] = breakpoints;
      console.log("breakpoints", this.breakpoints[this.file.name]);
      console.log("breakpoints", Object.entries(this.breakpoints));
    },

    deleteBreakpoint(fileName, line) {
      console.log("deleteBreakpoint", fileName, line);
      console.log("breakpoints", Object.entries(this.breakpoints))
      console.log("breakpoints2", fileName)
      console.log("breakpoints2", "code/src/lib.rs" === fileName)
      let breakpoints_old = [];
      for (let l of this.breakpoints[fileName]) {
        if (l === line) {
          console.log("breakpoint already exists -> removing");
          continue;
        }
        breakpoints_old.push(l);
      }
      this.breakpoints[fileName] = breakpoints_old;
      if (this.file.name === fileName) {
        this.breakpointsEditorRemove = line;
      }
      console.log("breakpoints", this.breakpoints);
    },

    load_file(name) {
      if (this.prev_node) {
        this.prev_node.open = false;
      }
      console.log("load_file", name);
      const name_split = name.split("/");
      this.tree.is_open = true;
      let node = this.tree;
      let count = 1;
      let run = true
      while (run) {
        for (var i of node["children"]) {
          if (i.name === name_split[count]) {
            i.is_open = true;
            count++;
            if (i.children.length === 0) {
              run = false;
            } 
            node = i;
            break;
          }
        }
      }
      node.open = true;
      this.prev_node = node;
      this.focus = !this.focus;
      this.breakpointsEditor = this.breakpoints[this.file.name];
    },
    
    // Tree
    changeFile(node) {
      if (this.prev_node) {
        this.prev_node.open = false;
      }
      console.log("APP changeFile", node.path);
      this.file = {"name": node.path };
      node.open = true;
      this.prev_node = node;
      this.breakpointsEditor = this.breakpoints[node.path];
    },
    toggleFolder(node) {
      console.log("APP toggleFolder", node);
      node.is_open = !node.is_open;
    },
  },
};
</script>

<style>
body {
  font-family: -apple-system,BlinkMacSystemFont,"Segoe UI","Noto Sans",Helvetica,Arial,sans-serif,"Apple Color Emoji","Segoe UI Emoji";
  font-size: 14px;
  background-color: #201c1c;
}

.add-item {
  color: #E0E4E6;
  background-color: transparent;
  border-radius: 6px;
  border-color: #30363d;
  border-style: solid;
  border-width: 1px;
  position: absolute;
  left: 20px;
  width: 30px;
  height: 30px;
  cursor: pointer;
  text-align: center;

}

.dis-view {
  height:100%;
  width:100%;
  overflow: scroll;
  background: #201c1c;
  border-color: #30363d;
  border-style: solid;
  border-radius: 6px;
  border-width: 1px;
}

.panel-header {
display: flex;
align-items: center;
flex-direction: row;
}

.delete-container {
    position: absolute;
    right: 2px;
    top: 0;
    cursor: pointer;
  }

.title {
margin-top: 0;
color: #E0E4E6;
font-weight: bold;
font-size: 1.2em;
width: 50px;
}

.switch-container {
margin-top: -8px;
margin-left: auto;
margin-right: 10px; 
}

.switch {
position: relative;
display: inline-block;
width: 25px;
height: 14px;
}

.switch input { 
opacity: 0;
width: 0;
height: 0;
}

.slider {
position: absolute;
cursor: pointer;
top: 0;
left: 0;
right: 0;
bottom: 0;
background-color: #98c379;
-webkit-transition: .4s;
transition: .4s;
}

.slider:before {
position: absolute;
content: "";
height: 9px;
width: 9px;
left: 3px;
bottom: 3px;
background-color: white;
-webkit-transition: .4s;
transition: .4s;
}

input:checked + .slider {
background-color: #e06c75;
}

input:checked + .slider:before {
-webkit-transform: translateX(10px);
-ms-transform: translateX(10px);
transform: translateX(10px);
}

.slider.round {
border-radius: 24px;
}

.slider.round:before {
border-radius: 50%;
}

.wrap {
  top: 0;
  left: 0;
  z-index: 15;
  padding: 5px;
  position: fixed;
  text-align: center;
  width: 100%;
  border: none;
  background-color: transparent;
  color: white;
  font-size: 14px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica,
    Arial, sans-serif;
}

.tx-hash-input {
  display: inline-block;
  margin-left: auto;
  margin-right: auto;
  text-align: left;
}

.content {
  margin-top: 40px;
}

.vue-grid-layout {
  background: #201c1c;
  left: 0;
}

.columns {
  -moz-columns: 120px;
  -webkit-columns: 120px;
  columns: 120px;
}

.vue-resizable-handle {
  z-index: 5000;
  position: absolute;
  width: 20px;
  height: 20px;
  bottom: 0;
  right: 0;
  background: url("data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/Pg08IS0tIEdlbmVyYXRvcjogQWRvYmUgRmlyZXdvcmtzIENTNiwgRXhwb3J0IFNWRyBFeHRlbnNpb24gYnkgQWFyb24gQmVhbGwgKGh0dHA6Ly9maXJld29ya3MuYWJlYWxsLmNvbSkgLiBWZXJzaW9uOiAwLjYuMSAgLS0+DTwhRE9DVFlQRSBzdmcgUFVCTElDICItLy9XM0MvL0RURCBTVkcgMS4xLy9FTiIgImh0dHA6Ly93d3cudzMub3JnL0dyYXBoaWNzL1NWRy8xLjEvRFREL3N2ZzExLmR0ZCI+DTxzdmcgaWQ9IlVudGl0bGVkLVBhZ2UlMjAxIiB2aWV3Qm94PSIwIDAgNiA2IiBzdHlsZT0iYmFja2dyb3VuZC1jb2xvcjojZmZmZmZmMDAiIHZlcnNpb249IjEuMSINCXhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeG1sbnM6eGxpbms9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxpbmsiIHhtbDpzcGFjZT0icHJlc2VydmUiDQl4PSIwcHgiIHk9IjBweCIgd2lkdGg9IjZweCIgaGVpZ2h0PSI2cHgiDT4NCTxnIG9wYWNpdHk9IjAuMzAyIj4NCQk8cGF0aCBkPSJNIDYgNiBMIDAgNiBMIDAgNC4yIEwgNCA0LjIgTCA0LjIgNC4yIEwgNC4yIDAgTCA2IDAgTCA2IDYgTCA2IDYgWiIgZmlsbD0iIzAwMDAwMCIvPg0JPC9nPg08L3N2Zz4=");
  background-position: bottom right;
  padding: 0 3px 3px 0;
  background-repeat: no-repeat;
  background-origin: content-box;
  box-sizing: border-box;
  cursor: se-resize;
}

/*.vue-grid-item:not(.vue-grid-placeholder) {
    border: 1px solid rgb(80, 80, 80);
}*/

.vue-grid-item.resizing {
  opacity: 0.9;
}

.vue-grid-item {
  background: transparent;
}

.vue-grid-item .text {
  font-size: 24px;
  text-align: center;
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  right: 0;
  margin: auto;
  height: 24px;
}

.vue-grid-item .minMax {
  font-size: 12px;
}

.vue-grid-item .add {
  cursor: pointer;
}

.remove {
  position: absolute;
  right: 2px;
  top: 0;
  cursor: pointer;
}
</style>
