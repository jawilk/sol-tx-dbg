<template>
    <div>
        <div>
            
        </div>
        <div class="content">
            <grid-layout v-model:layout="layout"
                         :col-num="12"
                         :row-height="30"
                         :is-draggable="true"
                         :is-resizable="true"
                         :vertical-compact="true"
                         :use-css-transforms="true">
                <grid-item v-for="item in layout" :key="item.i"
                           :x="item.x"
                           :y="item.y"
                           :w="item.w"
                           :h="item.h"
                           :i="item.i"
                           :is-resizable="item.isResizable">
                <component :node="data" :next="next"
                  @make-folder="$emit('make-folder', $event)"
                  @add-item2="$emit('add-item2', $event)" 
                  @next="next=!next"
                  :fileName="fileName" @change-file="changeFile"
                  @add-item="addItem" v-if="item.isComponent" 
                  :is="item.c"></component>
                       <div v-else v-html="item.c"></div>
                </grid-item>
            </grid-layout>
        </div>
    </div>
</template>

<script>
import AddBtn from './components/AddBtn.vue';
import EditorComponent from './components/Editor.vue';
import DebugPanel from './components/DebugPanel.vue';
// import Vue from 'vue';
import TreeNode from './components/TreeNode.vue';


const startLayout = [
    // {"x":6,"y":0,"w":1,"h":3,"i":"0","c":'AddBtn','isResizable':false,'isComponent': true},
    {"x":2,"y":1,"w":5,"h":15,"i":"1","c":'EditorComponent','isResizable':true,'isComponent': true},
    {"x":2,"y":0,"w":2,"h":1,"i":"2","c":'DebugPanel','isResizable':false,'isComponent': true},
    {"x":0,"y":0,"w":1,"h":10,"i":"3","c": 'TreeNode','isResizable':true,'isComponent': true},

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
//           {
//             name: "child folder 2",
//             children: [
             
//               { name: "hello2" },
//               { name: "wat2" },
            
//             ]
//           }
//         ]
//       };


export default {
    name: 'App',
    components: {
        AddBtn,
        EditorComponent,
        DebugPanel,
        TreeNode
    },
    data() {
        return {
            layout: JSON.parse(JSON.stringify(startLayout)),
            index: 3,
            fileName: '',
            next: true,
            data: {},
            prev_node: null,
        }
    },
    mounted() {
        this.index = this.layout.length;
        this.getTree();
    },
    methods: {
        async getTree() {
         const res = await fetch('http://localhost:8002/test.json')
         const finalRes = await res.json();
         this.data = finalRes;
         console.log("APP getTree", this.data)
        },
    changeFile(node) {
        if (this.prev_node) {
            this.prev_node.open = false;
        }
        console.log("APP changeFile", node.path)
        this.fileName = node.path;
        node.open = true;
        this.prev_node = node;
    },
        removeItem(i) {
            console.log("### REMOVE " + i);
            const index = this.layout.map(item => item.i).indexOf(i);
            this.layout.splice(index, 1);
        },
        addItem() {
            const item = {'isComponent': false,'isResizable':true,"x":0,"y":0,"w":2,"h":2,"i":this.index+"", 'c': ' <span class="remove" @click="removeItem(item.i)">x</span>'};
            this.index++;
            this.layout.push(item);
        },
        
    },
}
</script>

<style>
body {
  font: 14px -apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,Helvetica,Arial,sans-serif;
  background-color: #14171A;

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
    background: url('data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/Pg08IS0tIEdlbmVyYXRvcjogQWRvYmUgRmlyZXdvcmtzIENTNiwgRXhwb3J0IFNWRyBFeHRlbnNpb24gYnkgQWFyb24gQmVhbGwgKGh0dHA6Ly9maXJld29ya3MuYWJlYWxsLmNvbSkgLiBWZXJzaW9uOiAwLjYuMSAgLS0+DTwhRE9DVFlQRSBzdmcgUFVCTElDICItLy9XM0MvL0RURCBTVkcgMS4xLy9FTiIgImh0dHA6Ly93d3cudzMub3JnL0dyYXBoaWNzL1NWRy8xLjEvRFREL3N2ZzExLmR0ZCI+DTxzdmcgaWQ9IlVudGl0bGVkLVBhZ2UlMjAxIiB2aWV3Qm94PSIwIDAgNiA2IiBzdHlsZT0iYmFja2dyb3VuZC1jb2xvcjojZmZmZmZmMDAiIHZlcnNpb249IjEuMSINCXhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeG1sbnM6eGxpbms9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxpbmsiIHhtbDpzcGFjZT0icHJlc2VydmUiDQl4PSIwcHgiIHk9IjBweCIgd2lkdGg9IjZweCIgaGVpZ2h0PSI2cHgiDT4NCTxnIG9wYWNpdHk9IjAuMzAyIj4NCQk8cGF0aCBkPSJNIDYgNiBMIDAgNiBMIDAgNC4yIEwgNCA0LjIgTCA0LjIgNC4yIEwgNC4yIDAgTCA2IDAgTCA2IDYgTCA2IDYgWiIgZmlsbD0iIzAwMDAwMCIvPg0JPC9nPg08L3N2Zz4=');
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
    background: #282c34;
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
