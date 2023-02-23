<template>
    <li style="list-style-type: none">
      <div :class="{nodeheader: isFolder}" id="item" @click="toggle">
        <span ref="nodeFocus" v-if="isInEditor" class="curfile">-> </span>
        <span :class="{curfile: isInEditor}">{{ node.name }}</span>
        <span v-if="isFolder">[{{ node.is_open ? '-' : '+' }}]</span>
      </div>
      <ul v-show="node.is_open" v-if="isFolder">
        <TreeNode
           class="tree-node"
           v-for="(child, index) in node.children"
           :is-top-level="false"
           :key="index"
           :node="child"
           :focus="focus"
           @changeFile="$emit('changeFile', $event)"
           @toggleFolder="$emit('toggleFolder', $event)"
           @click="click(child)"
           >
      </TreeNode>
      </ul>
    </li>
  </template>

  <script>

export default {
  name: 'TreeNode',
  props: {'node': Object, 
          'focus': Boolean, 
          'isTopLevel': {
      type: Boolean,
      default: true
    }},
  computed: {
    isFolder() {
      return this.node.children && this.node.children.length;
    },
    isInEditor() {
      return this.node.open;
    }
  },
  watch: {
    node() {
      console.log("HEREEE")
    },
    focus() {
      this.$nextTick(function(){
        if (this.node.open) {
          this.$refs.nodeFocus.scrollIntoView({ behavior: 'smooth' });
      }
    });
  }
  },
  methods: {
    toggle() {
      console.log("TOGGLE", this.node)
      if (this.isFolder) {
        this.$emit('toggleFolder', this.node);
      }
    },
    click(node) {
        if (node.name.includes(".rs")) {
            console.log("clicked", node.name, node.path)
            this.$emit('changeFile', node);
        }
    }
  }
}

  </script>

  <style scoped>
  #item:hover {
    background-color: #2c3e50;
  }
  #item:after {
    background-color: yellow;
  }  .tree-node {
    cursor: pointer;
    color: #E0E4E6;
    overflow: hidden;
    border-style: none;
    font-size: 11px;
  }

  .nodeheader {
    font-weight: bold;
    color: #E0E4E6;
    cursor: pointer;
  }

  ul {
    padding-left: 1em;
    line-height: 1.5em;
    list-style-type: none;
  }

  .curfile {
    font-weight:bold;
    color: #98c379;
  }
    </style>