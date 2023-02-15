<template>
    <div class="tree-view">
      <div v-if="isTopLevel" class="panel-header">
        <p class="title">files</p>
        <div class="switch-container" title="remove">
        <label class="switch">
          <input type="checkbox">
          <span class="slider round"></span>
        </label>
        </div>  
        </div>
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
           @change-file="$emit('change-file', $event)"
           @toggle-folder="$emit('toggle-folder', $event)"
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
        this.$emit('toggle-folder', this.node);
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
     background: #201c1c;
     border-color: #30363d;
     border-style: solid;
     border-radius: 6px;
     border-width: 1px;
  }
  .tree-node {
    cursor: pointer;
    color: #E0E4E6;
    overflow: hidden;
    border-style: none;
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