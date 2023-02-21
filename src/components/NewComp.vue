<template>
  <ul v-show="!isComponent">
    <li
      v-for="comp in components"
      :key="comp.i"
      class="choose-comp"
      @click="choseComponent(comp.name)"
    >
      {{ comp.title }}
    </li>
  </ul>
</template>

<script>
// import TreeNode from "./TreeNode.vue";
// import DisassemblyComp from "./DisassemblyComp.vue";
// import BreakpointComp from "./BreakpointComp.vue";
// import LLDBComp from "./LLDBComp.vue";

const components = [
  {
    i: 0,
    title: "files",
    name: "TreeNode",
  },
  {
    i: 1,
    title: "disassembly",
    name: "DisassemblyComp",
  },
  {
    i: 2,
    title: "breakpoints",
    name: "BreakpointComp",
  },
  {
    i: 3,
    title: "lldb command",
    name: "LLDBComp",
  },
];

export default {
  name: "NewComp",
  props: ["id"],
  data() {
    return {
      components: components,
    };
  },
  methods: {
    choseComponent(comp) {
      this.$emit("choseComp", this.id, comp);
    },
  },
};
</script>

<style scoped>
.choose-comp {
  color: #e0e4e6;
  font-size: 20px;
  font-weight: bold;
  cursor: pointer;
  list-style-type: none;
}
</style>