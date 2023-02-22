<template>
  <ul v-show="!isComponent">
    <li
      v-for="comp in components"
      :key="comp.i"
      class="choose-comp"
      @click="choseComponent(comp)"
      :class="{ 'deactivated': comp.name === 'CallGraphComp' }"
    >
      {{ comp.title }}
    </li>
  </ul>
</template>

<script>
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
  {
    i: 4,
    title: "registers",
    name: "RegistersComp",
  },
  {
    i: 5,
    title: "variables",
    name: "VariablesComp",
  },
  {
    i: 6,
    title: "call graph",
    name: "CallGraphComp",
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
      if (comp.name === "CallGraphComp") {
        return;
      }
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