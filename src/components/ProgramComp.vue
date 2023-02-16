<template>
    <div class="tx-hash-wrap">
      <div v-drag="{ handle: '#debugDragHandle' }">
        <DebugPanel
          :isActive="isActive"
          @next="next"
          @continue="continue_"
          @restart="restart"
          @stop="stop"
        />
      </div>
    </div>
    <div class="content">
      <grid-layout
        v-model:layout="layout"
        :col-num="12"
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
          <component
            :node="tree"
            @make-folder="$emit('make-folder', $event)"
            @add-item2="$emit('add-item2', $event)"
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
            v-if="item.isComponent"
            :is="item.c"
          ></component>
          <div v-else v-html="item.c"></div>
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
// import lldbModule from "../lldb";

const startLayout = [
  // {"x":6,"y":0,"w":1,"h":3,"i":"0","c":'AddBtn','isResizable':false,'isComponent': true},
  {
    x: 1,
    y: 1,
    w: 5,
    h: 20,
    i: "1",
    c: "EditorComponent",
    isResizable: true,
    isComponent: true,
  },
  // {"x":2,"y":0,"w":2,"h":1,"i":"2","c":'DebugPanel','isResizable':false,'isComponent': true},
  {
    x: 0,
    y: 0,
    w: 1,
    h: 10,
    i: "2",
    c: "TreeNode",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 6,
    y: 0,
    w: 3,
    h: 8,
    i: "3",
    c: "DisassemblyComp",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 6,
    y: 0,
    w: 3,
    h: 8,
    i: "4",
    c: "BreakpointComp",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 6,
    y: 3,
    w: 3,
    h: 8,
    i: "5",
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

let lldbInstance = null;

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
  },
    // beforeCreate() {
    //   new lldbModule({
    //     locateFile(path) {
    //       if (path.endsWith(`.wasm`)) {
    //         return "http://localhost:8003/lldb.wasm";
    //       }
    //       return path;
    //     },
    //   }).then((myModule) => {
    //     lldbInstance = myModule;
    //     console.log("LOADED WASM");
    //   });
    // },
  data() {
    return {
      layout: JSON.parse(JSON.stringify(startLayout)),
      index: 3,
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
    };
  },
  async mounted() {
    this.index = this.layout.length;
    let hash_id = this.$route.query.hash_id;
    // const response = await fetch("http://localhost:8000/program/" + hash_id);
    this.getTree(hash_id.split('_')[1]);
    // await this.loadElf(hash_id.split('_')[1]);
    // await this.connect();
  },
  methods: {
    // LLDB commands
    async loadELf(program_id) {
      var data = await fetch("http://localhost:8003/" + program_id);
      data = await data["arrayBuffer"]();
      console.log(data);
      lldbInstance.FS.writeFile("program.so", new Uint8Array(data));
      lldbInstance.ccall("create_target", null, ["string"], ["program.so"]);
      // await lldbInstance.ccall('execute_command', 'number', ['string'], ['b process_instruction']);
    },
    async connect() {
      await lldbInstance.ccall('execute_command', 'number', ['string'], ['gdb-remote 9007']);
    },
    async executeLLDBCommand(command) {
      console.log("executeLLDBCommand", command);
      let ret = await lldbInstance.ccall("execute_command", "string", ["string"], [command]);
      this.lldbOutput = ret;
      console.log("executeLLDBCommand", ret);
    },
    // Debug Panel
    async next() {
      this.isActive = false;
      console.log("next");
      // await lldbInstance.ccall("request_next", null, [], []);
      await lldbInstance.ccall("execute_command", "string", ["string"], ['next']);
      await this.updateEditor();
      this.isActive = true;
    },
    async continue_() {
      this.isActive = false;
      console.log("continue");
      await lldbInstance.ccall("request_continue", null, [], []);
      this.updateEditor();
      this.isActive = true;
    },
    async restart() {
      console.log("restart");
      await lldbInstance.ccall("request_next", null, [], []);
      this.updateEditor();
    },
    async stop() {
      console.log("stop");
      await lldbInstance.ccall("request_next", null, [], []);
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
      let rxPtr = await lldbInstance._malloc(request.length + 1);
      await lldbInstance.stringToUTF8(request, rxPtr, request.length + 1);
      const txPtr = await lldbInstance.ccall(
        "request_stackTrace",
        "number",
        ["number"],
        [rxPtr],
      );
      const responseStr = await lldbInstance.UTF8ToString(txPtr);
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
      let res = await lldbInstance.ccall('execute_command', 'string', ['string'], ['disassemble -p -b -c 7']);
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
      if (fileName.includes('solana-program-1.10.35')) {
        fileName = 'code/sdk/program/' + fileName.split('solana-program-1.10.35/')[1]
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

.tx-hash-wrap {
  top: 0;
  left: 0;
  z-index: 15;
  padding: 5px;
  position: fixed;
  text-align: center;
  width: 100%;
  border: none;
  background-color: #201c1c;
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
