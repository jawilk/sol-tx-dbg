<template>
  <div class="wrap" :class="{ dragging: isDragging }">
    <div v-drag="{ handle: '#debugDragHandle' }">
      <DebugPanel
        :isActive="isActive"
        :isRestart="isRestart"
        @stepIn="stepIn"
        @stepOut="stepOut"
        @next="next"
        @continue="continue_"
      />
    </div>
    <span class="add-wrap">
      <button class="add-item" @click="addPanel" title="add panel">+</button>
    </span>
    <p
      class="program-status"
      :class="{
        'status-running': status === 'Running' || status === 'Running (CPI)',
        'status-in-cpi': status === 'In CPI',
        'status-finished': status === 'Finished',
      }"
    >
      {{ status }}
    </p>
    <p class="program-name" :title="program_id">{{ program_name }}</p>
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
        @move="onDragStart"
        @moved="onDragStop"
        :class="{ dragging: isDragging }"
        v-for="item in layout"
        :key="item.i"
        :x="item.x"
        :y="item.y"
        :w="item.w"
        :h="item.h"
        :i="item.i"
        :is-resizable="item.isResizable"
        :drag-allow-from="
          item.name !== 'editor' ? '.vue-draggable-handle' : null
        "
      >
        <div class="dis-view">
          <div v-if="item.name !== 'editor'" class="panel-header">
            <p class="title">{{ item.name }}</p>
            <div title="drag me" class="vue-draggable-handle">&harr;</div>
            <div class="delete-container" title="remove">
              <span class="delete" @click="removePanel(item.i)">x</span>
            </div>
          </div>
          <component
            v-bind="getProps(item.comp, item.i)"
            v-on="getListeners(item.comp)"
            v-if="item.isComponent"
            :is="item.comp"
          ></component>
        </div>
      </grid-item>
    </grid-layout>
  </div>
</template>

<script>
import * as bs58 from "bs58";
import AddBtn from "./AddBtn.vue";
import EditorComponent from "./Editor.vue";
import DebugPanel from "./DebugPanel.vue";
import TreeNode from "./TreeNode.vue";
import DisassemblyComp from "./DisassemblyComp.vue";
import BreakpointComp from "./BreakpointComp.vue";
import LLDBComp from "./LLDBComp.vue";
import NewComp from "./NewComp.vue";
import lldbModule from "../lldb";
import RegistersComp from "./RegistersComp.vue";
import VariablesComp from "./VariablesComp.vue";
import MemoryComp from "./MemoryComp.vue";

const startLayout = [
  {
    x: 1,
    y: 0,
    w: 5,
    h: 20,
    i: "0",
    name: "editor",
    comp: "EditorComponent",
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
    comp: "TreeNode",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 6,
    y: 1,
    w: 3,
    h: 6,
    i: "2",
    name: "disassembly",
    comp: "DisassemblyComp",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 9,
    y: 1,
    w: 3,
    h: 6,
    i: "3",
    name: "breakpoints",
    comp: "BreakpointComp",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 8,
    y: 2,
    w: 3,
    h: 8,
    i: "4",
    name: "lldb command",
    comp: "LLDBComp",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 6,
    y: 2,
    w: 2,
    h: 9,
    i: "5",
    name: "registers",
    comp: "RegistersComp",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 6,
    y: 0,
    w: 6,
    h: 7,
    i: "6",
    name: "variables",
    comp: "VariablesComp",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 1,
    y: 3,
    w: 4,
    h: 8,
    i: "7",
    name: "memory map",
    comp: "MemoryComp",
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
    RegistersComp,
    VariablesComp,
    MemoryComp,
  },
  data() {
    return {
      files_url: "http://localhost:8000/static/",
      websocket_url: "ws://localhost:9007/?token=",
      cpi_url: "http://localhost:8083/program/",
      LLDB: null,
      layout: startLayout,
      index: 0,
      colNum: 12,
      editorState: {},
      breakpoints: {},
      lineMark: {},
      tree: {},
      prev_node: null,
      seqId: 0,
      focus: false,
      status: "",
      isDebuggerConnected: false,
      isActive: false,
      isRestart: false,
      disData: [],
      uuid: "",
      tx_hash: "",
      inst_nr: 0,
      program_id: "",
      registers: [],
      variables: [],
      program_name: "",
      program_real_path: "/home/wj/temp/test-solana/program/",
      isDragging: false,
      programs_supported: [],
    };
  },
  beforeCreate() {
    window.addEventListener("beforeunload", this.handleWindowClose);
  },
  unmounted() {
    window.removeEventListener("beforeunload", this.handleWindowClose);
  },
  async mounted() {
    console.log("program query", this.$route.query);
    // vue-grid-layout
    this.index = this.layout.length;
    // instantiate lldb.wasm
    this.LLDB = await this.fetchLLDB();
    // query params
    this.uuid = this.$route.query.uuid;
    this.program_id = this.$route.query.program_id;
    if (this.$route.query.program_name) {
      this.program_name = this.$route.query.program_name;
    } else {
      this.program_name = this.program_id;
    }
    console.log("name", this.program_name);
    if (!this.$route.query.tx_hash) {
      console.log("IS CPI");
      this.status = "Starting CPI...";
      this.LLDB["websocket"]["url"] = this.websocket_url + this.uuid;
    } else {
      console.log("NO CPI");
      this.status = "Starting...";
      this.tx_hash = this.$route.query.tx_hash;
      this.inst_nr = this.$route.query.inst_nr;
      this.LLDB["websocket"]["url"] =
        this.websocket_url +
        this.uuid +
        "&tx_hash=" +
        this.tx_hash +
        "&inst_nr=" +
        this.inst_nr;
    }
    await this.getTree();
    await this.loadElf();
    // CPI hook
    let res = await this.LLDB.ccall(
      "execute_command",
      "number",
      ["string"],
      ["b solana_program::program::invoke_signed_unchecked"]
    );
    await this.LLDB._free(res);
    // connect to vm
    await this.connect();
    fetch(this.files_url + "programs_supported.txt")
      .then((response) => response.text())
      .then((text) => {
        // split the text by line breaks and store each line in an array
        const lines = text.split("\n");
        this.programs_supported = lines;
      })
      .catch((error) => console.error(error));
    console.log("programs_supported", this.programs_supported);
    this.isDebuggerConnected = true;
    this.isActive = true;
    console.log("mounted");
    if (this.status === "Starting CPI...") {
      this.status = "Running (CPI)";
    } else {
      this.status = "Running";
    }
    // update panels
    await this.updateEditor();
  },
  methods: {
    // To prevent text selection while dragging
    onDragStart() {
      this.isDragging = true;
      console.log("drag start", this.isDragging);
    },
    onDragStop() {
      this.isDragging = false;
      console.log("drag stop", this.isDragging);
    },
    handleWindowClose() {
      if (!this.isDebuggerConnected) {
        const ws = new WebSocket(this.websocket_url + this.uuid);
        // ws.send('close');
        ws.close();
      }
    },
    cleanup() {
      this.LLDB.exports = null;
      this.LLDB = null;
    },
    getProps(comp, id) {
      switch (comp) {
        case "EditorComponent":
          return {
            files_url: this.files_url,
            program_id: this.program_id,
            editorState: this.editorState,
          };
        case "TreeNode":
          return { node: this.tree, focus: this.focus };
        case "DisassemblyComp":
          return { disData: this.disData };
        case "BreakpointComp":
          return { breakpoints: this.breakpoints };
        case "LLDBComp":
          return { executeLLDBCommand: this.executeLLDBCommand };
        case "MemoryComp":
          return { getMemory: this.getMemory };
        case "NewComp":
          return { id: id };
        case "RegistersComp":
          return { registers: this.registers };
        case "VariablesComp":
          return { variables: this.variables };
        default:
          return {};
      }
    },
    getListeners(comp) {
      switch (comp) {
        case "EditorComponent":
          return { ["toggleBreakpoints"]: this.toggleBreakpoints };
        case "TreeNode":
          return {
            ["changeFile"]: this.changeFile,
            ["toggleFolder"]: this.toggleFolder,
          };
        case "BreakpointComp":
          return { ["deleteBreakpoint"]: this.toggleBreakpoints };
        case "NewComp":
          return { ["choseComp"]: this.choseComp };
        default:
          return {};
      }
    },
    async fetchLLDB() {
      const wasm_url = this.files_url + "lldb.wasm";
      return new lldbModule({
        locateFile(path) {
          if (path.endsWith(`.wasm`)) {
            return wasm_url;
          }
          return path;
        },
      });
    },
    // Components
    removePanel(id) {
      const index = this.layout.findIndex((item) => item.i === id);
      this.layout.splice(index, 1);
    },
    addPanel() {
      this.layout.push({
        x: 2,
        y: 1,
        w: 3,
        h: 10,
        i: this.index,
        name: "add panel",
        comp: "NewComp",
        isResizable: true,
        isComponent: true,
      });
      this.index++;
    },
    choseComp(id, comp) {
      const index = this.layout.findIndex((item) => item.i === id);
      this.layout[index].comp = comp.name;
      this.layout[index].name = comp.title;
    },
    // LLDB commands
    async loadElf() {
      const file_name = this.program_id + ".so";
      var data = await fetch(this.files_url + "elfs/" + file_name);
      data = await data["arrayBuffer"]();
      this.LLDB.FS.writeFile(file_name, new Uint8Array(data));
      let res = this.LLDB.ccall("create_target", null, ["string"], [file_name]);
      this.LLDB._free(res);
    },
    async connect() {
      const res = await this.LLDB.ccall(
        "execute_command",
        "number",
        ["string"],
        ["gdb-remote 9007"]
      );
      this.LLDB._free(res);
    },
    async executeLLDBCommand(command) {
      if (!this.isActive) {
        return "please wait for the current action to finish or restart";
      }
      console.log("executeLLDBCommand", command);
      let resPtr = await this.LLDB.ccall(
        "execute_command",
        "number",
        ["string"],
        [command]
      );
      const lldbOutput = await this.LLDB.UTF8ToString(resPtr);
      this.LLDB._free(resPtr);
      return lldbOutput;
    },
    async getMemory(address, bytes) {
      if (!this.isActive) {
        return "please wait for the current action to finish or restart";
      }
      const command = "mem read " + address + " -c " + bytes;
      let resPtr = await this.LLDB.ccall(
        "execute_command",
        "number",
        ["string"],
        [command]
      );
      const lldbOutput = await this.LLDB.UTF8ToString(resPtr);
      this.LLDB._free(resPtr);
      return lldbOutput;
    },
    async handleCpi() {
      await this.updateEditor();
      const pubkeyArr = await this.LLDB.ccall(
        "request_cpi_program_id",
        "number",
        [],
        []
      );
      const pubkey = bs58.encode(
        new Uint8Array(this.LLDB.HEAPU8.buffer, pubkeyArr, 32)
      );
      console.log("request_cpi_program_id: ", pubkey);
      let url;
      if (!this.programs_supported.includes(pubkey)) {
        url = this.cpi_url + "not-supported?program_id=" + pubkey;
      } else {
        url = this.cpi_url + "?uuid=" + this.uuid + "&program_id=" + pubkey;
      }
      this.status = "In CPI";
      window.open(url);
      // This will block till cpi finished
      await this.LLDB.ccall("request_next_with_cpi", "boolean", [], []);
      this.status = "Running";
      console.log("CPI finished");
    },
    async check_finished() {
      const isFinished = await this.LLDB.ccall(
        "should_terminate",
        "number",
        [],
        []
      );
      if (isFinished === -1) {
        this.status = "Finished";
        console.log("execution finished");
        this.cleanup();
        alert("execution finished");
        if (this.status !== "In CPI") {
          this.isRestart = true;
          console.log("restart");
        }
        return;
      }
      await this.updateEditor();
      this.isActive = true;
    },
    // Debug Panel
    async stepIn() {
      this.isActive = false;
      console.log("request_stepIn_with_cpi");
      const is_before_cpi = await this.LLDB.ccall(
        "request_stepIn_with_cpi",
        "boolean",
        [],
        []
      );
      // CPI
      if (is_before_cpi === true) {
        await this.handleCpi();
      }
      await this.check_finished();
    },
    async stepOut() {
      this.isActive = false;
      console.log("request_stepOut_with_cpi");
      let resPtr = await this.LLDB.ccall(
        "execute_command",
        "number",
        ["string"],
        ["finish"]
      );
      this.LLDB._free(resPtr);
      await this.updateEditor();
      this.isActive = true;
    },
    async next() {
      this.isActive = false;
      console.log("request_next_with_cpi");
      const is_before_cpi = await this.LLDB.ccall(
        "request_next_with_cpi",
        "boolean",
        [],
        []
      );
      // CPI
      if (is_before_cpi === true) {
        await this.handleCpi();
      }
      await this.check_finished();
    },
    async continue_() {
      this.isActive = false;
      console.log("continue");
      const is_before_cpi = await this.LLDB.ccall(
        "request_continue_with_cpi",
        "boolean",
        [],
        []
      );
      // CPI
      if (is_before_cpi === true) {
        await this.handleCpi();
      }
      await this.check_finished();
    },
    // Update
    async LLDBRequest(request, name) {
      if (this.LLDB === null) {
        return;
      }
      const requestStr = JSON.stringify(request);
      console.log("LLDBRequest", requestStr);
      let txPtr = await this.LLDB._malloc(requestStr.length + 1);
      await this.LLDB.stringToUTF8(requestStr, txPtr, requestStr.length + 1);
      const rxPtr = await this.LLDB.ccall(name, "number", ["number"], [txPtr]);
      const responseStr = await this.LLDB.UTF8ToString(rxPtr);
      let responseJSON = JSON.parse(responseStr);
      console.log("response", responseJSON);
      this.seqId++;
      // Cleanup
      await this.LLDB._free(txPtr);
      await this.LLDB._free(rxPtr);
      return responseJSON;
    },
    async updateEditor() {
      console.log("update editor");
      var request = {
        type: "request",
        seq: this.seqId,
        command: "stackTrace",
        arguments: { threadId: 1, startFrame: 0, levels: 10 },
      };
      const responseJSON = await this.LLDBRequest(
        request,
        "request_stackTrace"
      );

      console.log("PATH:", responseJSON.body.stackFrames[0].source.path);
      let path = responseJSON.body.stackFrames[0].source.path;
      if (path) {
        let file = this.sanitizeFileName(path);
        this.load_file(file);
        this.editorState["file"] = file;
        this.editorState["line"] = responseJSON.body.stackFrames[0].line;
        this.lineMark["file"] = file;
        this.lineMark["line"] = responseJSON.body.stackFrames[0].line;
        let breakpointsEditor;
        if (this.breakpoints[this.editorState.file] === undefined) {
          breakpointsEditor = [];
        } else {
          breakpointsEditor = JSON.parse(
            JSON.stringify(this.breakpoints[this.editorState.file])
          );
        }
        this.editorState["breakpoints"] = breakpointsEditor;
        this.editorState = JSON.parse(JSON.stringify(this.editorState));
      } else {
        console.log("DISASSEMBLY");
      }
      await this.disassemble();
      await this.getRegisters();
      await this.getVariables();
      console.log("update editor done");
    },
    async getVariables() {
      var request = {
        type: "request",
        seq: this.seqId,
        command: "variables",
        arguments: { variablesReference: 1 },
      };
      const responseJSON = await this.LLDBRequest(request, "request_variables");

      this.variables = responseJSON.body.variables;
      console.log("variables", this.variables);
    },
    async getRegisters() {
      let resPtr = await this.LLDB.ccall(
        "execute_command",
        "number",
        ["string"],
        ["register read"]
      );
      let registers = this.LLDB.UTF8ToString(resPtr);
      let registersParsed = [];
      let regex = /(\w+)\s=\s(0x[\dA-Fa-f]+)/g;
      let match = regex.exec(registers);
      let count = 0;
      while (match !== null) {
        registersParsed.push({ id: count, name: match[1], value: match[2] });
        match = regex.exec(registers);
        count++;
      }
      console.log("reg", registersParsed);
      this.registers = registersParsed;
      this.LLDB._free(resPtr);
    },
    async disassemble() {
      let resPtr = await this.LLDB.ccall(
        "execute_command",
        "number",
        ["string"],
        ["disassemble -p -b -c 7"]
      );
      let res = this.LLDB.UTF8ToString(resPtr);
      res = res.split("\n").slice(1);
      res[0] = res[0].slice(3); // Remove leading arrow
      let data = [];
      for (let line of res) {
        try {
          let addr = line.split(" <")[0];
          let inst = line.split(": ")[1].slice(0, 23);
          let mnem = line.split(inst)[1];
          data.push({ addr: addr, inst: inst.toUpperCase(), mnem: mnem });
        } catch (e) {
          //console.log("error", e)
        }
      }
      this.disData = data;
      this.LLDB._free(resPtr);
    },
    sanitizeFileName(fileName) {
      if (fileName.includes("solana-program-1.10.33")) {
        fileName =
          "code/sdk/solana-program-1.10.33/" +
          fileName.split("solana-program-1.10.33/")[1];
      } else if (fileName.includes("rust-own")) {
        fileName =
          "code/rust-solana-1.59.0/" + fileName.split("rust-own/rust/")[1];
      } else {
        fileName = "code/program/" + fileName.split("/program/")[1];
      }
      console.log("sanitized:", fileName);
      return fileName;
    },
    // Setup
    async getTree() {
      const res = await fetch(
        this.files_url + "trees/" + this.program_id + ".json"
      );
      this.tree = await res.json();
    },
    sanitizeBreakpointPath(path) {
      if (path.includes("solana-program-1.10.33")) {
        path =
          "/home/wj/.cargo/registry/src/github.com-1ecc6299db9ec823/solana-program-1.10.33/" +
          path.split("solana-program-1.10.33/")[1];
      } else if (path.includes("rust-solana-1.59.0")) {
        path =
          "/home/wj/projects/rust-own/rust/" +
          path.split("rust-solana-1.59.0/")[1];
      } else {
        path = this.program_real_path + path.split("/program/")[1];
      }
      return path;
    },
    // Editor
    async toggleBreakpoints(file, line) {
      if (this.LLDB === null) {
        return;
      }
      console.log("toggleBreakpoints0", this.breakpoints[file]);
      if (file === "") {
        file = this.editorState.file;
      }
      let preBreakpoints;
      console.log("toggleBreakpoints", file, line);
      if (this.breakpoints[file] === undefined) {
        preBreakpoints = [];
      } else {
        preBreakpoints = this.breakpoints[file].slice();
      }
      const index = preBreakpoints.indexOf(line);
      if (index !== -1) {
        preBreakpoints.splice(index, 1);
      } else {
        preBreakpoints.push(line);
      }
      const breakpointsReq = preBreakpoints.map((num) => {
        return { line: num };
      });
      var request = {
        type: "request",
        seq: this.seqId,
        command: "setBreakpoints",
        arguments: {
          source: { path: this.sanitizeBreakpointPath(file) },
          breakpoints: breakpointsReq,
        },
      };
      const responseJSON = await this.LLDBRequest(
        request,
        "request_setBreakpoints"
      );
      console.log("responseJSON", responseJSON);
      this.breakpoints[file] = responseJSON.body.breakpoints
        .filter((b) => b.verified === true)
        .map((b) => b.line);

      if (file === this.editorState.file) {
        this.editorState.breakpoints = this.breakpoints[this.editorState.file];
        this.editorState = JSON.parse(JSON.stringify(this.editorState));
        console.log("editorState.breakpoints", this.editorState.breakpoints);
      }
      console.log("breakpoints", this.breakpoints[this.editorState.file]);
    },

    load_file(name) {
      if (name == this.editorState.file) {
        return;
      }
      if (this.prev_node) {
        this.prev_node.open = false;
      }
      console.log("load_file", name);
      const name_split = name.split("/");
      this.tree.is_open = true;
      let node = this.tree;
      let count = 1;
      let run = true;
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
    },

    // Tree
    changeFile(node) {
      if (this.prev_node) {
        this.prev_node.open = false;
      }
      console.log("APP changeFile", node.path);
      this.editorState["file"] = node.path;
      node.open = true;
      this.prev_node = node;
      console.log("changeFile breakpoints", this.breakpoints);
      this.editorState["breakpoints"] = this.breakpoints[node.path];
      if (node.path === this.lineMark.file) {
        this.editorState["line"] = this.lineMark.line;
      } else {
        this.editorState["line"] = undefined;
      }
      this.editorState = JSON.parse(JSON.stringify(this.editorState));
    },
    toggleFolder(node) {
      console.log("APP toggleFolder", node);
      node.is_open = !node.is_open;
    },
  },
};
</script>

<style>
.add-wrap {
  position: absolute;
  background: linear-gradient(90deg, #9945ff 38.82%, #9945ff 0, #14f195 64.82%);
  padding: 1px;
  border-radius: 6px;
  border-width: 1em;
  left: 20px;
  width: 30px;
  height: 30px;
}

.add-item {
  color: #e0e4e6;
  background-color: #201c1c;
  border-radius: 6px;
  border-color: #30363d;
  border-style: solid;
  border-width: 1px;
  width: 100%;
  height: 100%;
  cursor: pointer;
  text-align: center;
  font-size: 15px;
}

.program-status {
  color: #e0e4e6;
  font-size: 16px;
  background-color: transparent;
  position: absolute;
  left: 40px;
  width: 200px;
  height: 30px;
  top: 0;
  text-align: center;
}

.status-finished {
  color: red;
}

.status-in-cpi {
  color: orange;
}

.status-running {
  color: #14f195;
}

.program-name {
  color: #e0e4e6;
  font-size: 16px;
  background-color: transparent;
  position: absolute;
  left: 200px;
  width: 200px;
  height: 30px;
  top: 0;
  text-align: center;
}

.dis-view {
  height: 100%;
  width: 100%;
  overflow: scroll;
  background: #201c1c;
  border-color: #30363d;
  border-style: solid;
  border-radius: 6px;
  border-width: 1px;
}

.panel-header {
  display: flex;
  flex-direction: row;
  align-items: center;
}

.title {
  margin-top: 0;
  color: #e0e4e6;
  font-weight: bold;
  font-size: 14px;
  width: 150px;
}

.vue-draggable-handle {
  position: absolute;
  color: #e0e4e6;
  font-size: 20px;
  top: -10px;
  right: 50%;
  cursor: pointer;
}

.delete-container {
  position: absolute;
  right: 10px;
  top: 0;
  cursor: pointer;
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

.vue-grid-item.resizing {
  opacity: 0.9;
}

.vue-grid-item {
  background: transparent;
}

.vue-grid-item .text {
  font-size: 14px;
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

.dragging {
  user-select: none;
}

.remove {
  position: absolute;
  right: 2px;
  top: 0;
  cursor: pointer;
}
</style>
