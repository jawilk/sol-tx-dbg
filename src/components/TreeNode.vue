<template>
    <div class="tree-view">
    <li style="list-style-type: none">
      <div :class="{nodeheader: isFolder}" id="item" @click="toggle">
        <span v-if="isInEditor">-></span>
        {{ node.name }}
        <span v-if="isFolder">[{{ isOpen ? '-' : '+' }}]</span>
      </div>
      <ul v-show="isOpen" v-if="isFolder">
        <TreeNode
           class="tree-node"
           v-for="(child, index) in node.children"
           :key="index"
           :node="child"
           @change-file="$emit('change-file', $event)"
           @click="click(child)"
           >
      </TreeNode>
      </ul>
    </li>
</div>
  </template>

  <script>

export default {
    name: 'TreeNode',
    props: ['node'],
  data() {
    return {
      isOpen: false,
    };
  },
  watch: {
    isClicked() {
      console.log("clifgfgcked", this.isClicked)
  }
  },
  computed: {
    isFolder() {
      return this.node.children && this.node.children.length;
    },
    isInEditor() {
      return this.node.open;
    }
  },
  methods: {
    toggle: function() {
      if (this.isFolder) {
        this.isOpen = !this.isOpen;
      }
    },
    click(node) {
        if (node.name.includes(".rs")) {
            console.log("clicked", node.name, node.path)
            this.$emit('change-file', node);
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
  }
  .tree-view {
    height:100%;
     width:100%;
     overflow: scroll;
  }
  .tree-node {
    cursor: pointer;
    color: white;
  }

  .nodeheader {
    font-weight: bold;
    color: white;
  }

  ul {
    padding-left: 1em;
    line-height: 1.5em;
    list-style-type: none;
  }
    </style>