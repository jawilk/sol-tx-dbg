<template>
    <component
    v-show="isComponent"
    :is="component"
    :node="node"
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
  ></component>
  <ul v-show="!isComponent">
    <li v-for="comp in components" :key="comp.i" class="choose-comp" @click="choseComponent(comp.c, comp.title)">
        {{ comp.title }}
    </li>
    </ul>
</template>

<script>
import TreeNode from "./TreeNode.vue";
import DisassemblyComp from "./DisassemblyComp.vue";
import BreakpointComp from "./BreakpointComp.vue";
import LLDBComp from "./LLDBComp.vue";

const components = [
  {
    x: 0,
    y: 0,
    w: 1,
    h: 10,
    i: "1",
    title: "files",
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
    title: "disassembly",
    c: "DisassemblyComp",
    isResizable: true,
    isComponent: true,
  },
  {
    x: 6,
    y: 0,
    w: 3,
    h: 8,
    i: "3",
    title: "breakpoints",
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
    title: "lldb command",
    c: "LLDBComp",
    isResizable: true,
    isComponent: true,
  },
];

export default {
  name: 'NewComp',
  props: ['breakpoints', 'breakpointsEditor', 'breakpointsEditorRemove', 'lldbOutput', 'disData', 'line', 'file', 'focus', 'node'],
  components: {
    TreeNode,
    DisassemblyComp,
    BreakpointComp,
    LLDBComp,
  },
  data() {
    return {
      components: components,
      isComponent: false,
      component: null,
    };
  },
  methods: {
    choseComponent(comp, name) {
        this.isComponent = true;
        this.component = comp;
        this.$emit('changeName', name);
    },
    deleteBreakpoint(file, line) {
        this.$emit('deleteBreakpoint', file, line);
    },
    changeFile(node) {
        this.$emit('change-file', node);
    },
    toggleFolder(node) {
        this.$emit('toggle-folder', node);
    },
  }
}
</script>

<style scoped>
.choose-comp {
  color: #E0E4E6;
  font-size: 20px;
  font-weight: bold;
  cursor: pointer;
  list-style-type: none;
}
</style>