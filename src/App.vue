<template>
  <div>
    <div class="tx-hash-wrap">
      <form class="tx-hash-input">
        <input
          style="width: 400px"
          key="input"
          type="text"
          placeholder="tx hash"
        />
        <input type="submit" value="go" style="cursor: pointer" />
      </form>
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
        <div v-drag="{ handle: '#debugDragHandle' }">
          <DebugPanel
            @next="next"
            @continue="continue_"
            @restart="restart"
            @stop="stop"
          />
        </div>
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
            @change-file="changeFile"
            @toggle-folder="toggleFolder"
            @add-item="addItem"
            v-if="item.isComponent"
            :is="item.c"
          ></component>
          <div v-else v-html="item.c"></div>
        </grid-item>
      </grid-layout>
    </div>
  </div>
</template>

<script>
import AddBtn from "./components/AddBtn.vue";
import EditorComponent from "./components/Editor.vue";
import DebugPanel from "./components/DebugPanel.vue";
import TreeNode from "./components/TreeNode.vue";
import lldbModule from "./lldb";
// import { DebugProtocol } from '@vscode/debugprotocol';

const startLayout = [
  // {"x":6,"y":0,"w":1,"h":3,"i":"0","c":'AddBtn','isResizable':false,'isComponent': true},
  {
    x: 1,
    y: 1,
    w: 5,
    h: 15,
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
    i: "3",
    c: "TreeNode",
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
  },
    beforeCreate() {
      new lldbModule({
        locateFile(path) {
          if (path.endsWith(`.wasm`)) {
            return "http://localhost:8003/lldb.wasm";
          }
          return path;
        },
      }).then((myModule) => {
        lldbInstance = myModule;
        console.log("LOADED");
        this.callAdd();
      });
    },
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
    };
  },
  mounted() {
    this.index = this.layout.length;
    this.getTree();
  },
  methods: {
    // Debug Panel
    async next() {
      console.log("next");
      await lldbInstance.ccall("request_next", null, [], []);
      await this.updateEditor();
    },
    async continue_() {
      console.log("continue");
      await lldbInstance.ccall("request_continue", null, [], []);
      this.updateEditor();
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
      let file = this.sanitizeFileName(responseJSON.body.stackFrames[0].source.path);
      if (this.file.name !== file) {
        this.file.name = file;
        this.file.line = responseJSON.body.stackFrames[0].line;
        this.load_file(file);
        this.file = {"name": file, "line": responseJSON.body.stackFrames[0].line}
      } else {
      this.line = responseJSON.body.stackFrames[0].line;
      }
      // let res = await lldbInstance.ccall('execute_command', 'string', ['string'], ['frame info']);
      // console.log("threads", res);
    },

    sanitizeFileName(fileName) {
      if (fileName.includes('solana-program-1.10.35')) {
        fileName = 'code/sdk/program/' + fileName.split('solana-program-1.10.35/')[1]
      }
      console.log("sanitized:", fileName)
      return fileName;
    },

    // Setup
    async getTree() {
      const res = await fetch("http://localhost:8003/test.json");
      const finalRes = await res.json();
      this.tree = finalRes;
      console.log("APP getTree", this.tree);
      this.load_file("code/src/lib.rs");
      this.file = {"name": "code/src/lib.rs", line: 1};
    },
    async callAdd() {
      const abc = lldbInstance.ccall(
        "execute_command",
        "string",
        ["string"],
        ["version"]
      );
      console.log(abc);
      var data = await fetch("http://localhost:8003/hello.so");
      data = await data["arrayBuffer"]();
      console.log(data);
      lldbInstance.FS.writeFile("hello.so", new Uint8Array(data));
      lldbInstance.ccall("create_target", null, ["string"], ["hello.so"]);
      const target = lldbInstance.ccall(
        "execute_command",
        "string",
        ["string"],
        ["target list"]
      );
      console.log(target);
      await lldbInstance.ccall('execute_command', 'number', ['string'], ['gdb-remote 9007']);
    },
    // Editor
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
      this.focus = ~this.focus;
      // for (var i of this.tree["children"]) {
      //   console.log(i);
      //   if (i.name === name_split[1]) {
      //     i.is_open = true;
      //     for (var j of i["children"]) {
      //       if (j.name === name_split[2]) {
      //         j.open = true;
      //       }
      //     }
      //   }
      // }
      // this.tree['children'][4]['children'][0].open = true;
      // this.prev_node = this.tree["children"][4]["children"][0];
      // this.fileName = name;
    },
    changeFile(node) {
      if (this.prev_node) {
        this.prev_node.open = false;
      }
      console.log("APP changeFile", node.path);
      this.file = {"name": node.path };
      node.open = true;
      this.prev_node = node;
    },
    // Tree
    toggleFolder(node) {
      console.log("APP toggleFolder", node);
      node.is_open = !node.is_open;
    },
  },
};
</script>

<style>
body {
  font: 14px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica,
    Arial, sans-serif;
  background-color: #14171a;
}

.tx-hash-wrap {
  display: block;
  text-align: center;
  width: 100%;
  border: none;
  background-color: #14171a;
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

.vue-grid-layout {
  background: black;
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
