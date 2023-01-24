<template>
<div id="treeView">
  <tree  @node-focus="focus" :nodes="nodes" :config="config"></tree>
</div>
</template>


<script>
import treeview from "vue3-treeview";
import "vue3-treeview/dist/style.css";

import { parseConfig, parseNodes } from "../get_config.js";

// async parseFile(url) {
//    try {
//      const fetchResponse = await fetch(
//        url
//      );
//      console.log("HERE")
//       return fetchResponse;
//    } catch (ex) {
//      console.log("Error in fetch");
//    }
// };

export default {
  name: "TreeView",
  components: {
    tree: treeview,
  },
  data() {
    return {
      config: {roots: parseConfig()},
      nodes: parseNodes(),
      // config: {
      //   roots: ["id1", "id2"],
      // },
    //   nodes: {
    //     id1: {
    //       text: "text1",
    //       children: ["id11", "id12"],
    //     },
    //     id11: {
    //       text: "text11",
    //     },
    //     id12: {
    //       text: "text12",
    //     },
    //     id2: {
    //       text: "text2",
    //     },
    //   },
    };
  },
  // setup() {
    // fetch('http://localhost:8001/test.json').then(response => response.json())
      // .then(txt => {
        // console.log(txt.config)
      // const config = {
      //   roots: ["id1", "id2"],
      // };
      // const nodes = {
      //   id1: {
      //     text: "text1434",
      //     children: ["id11", "id12"],
      //   },
      //   id11: {
      //     text: "text11",
      //   },
      //   id12: {
      //     text: "text12",
      //   },
      //   id2: {
      //     text: "text2",
      //   },
      // };
      // return {
      //   // config,
      //   nodes
      // }
  // })
  // },
  methods: {
  //     async parseFile(url) {
  //  try {
  //    const fetchResponse = await fetch(
  //      url
  //    );
  //    console.log("HERE")
  //     return fetchResponse;
  //  } catch (ex) {
  //    console.log("Error in fetch");
  //  }
  //     },
    focus(node) {
            if (node.children.length === 0) {
                console.log("childless node: " + node.text)
                this.$emit('change-file', node.text);
        }
  }
  }
};

</script>

<style>

#treeView {
  width: 100%;
  height: 100%;
  background-color: #282c34;
}

.node-text {
  color: white;
  font-size: 14px;
}

.node-wrapper:hover {
  background-color: #3e4451;
}

.node-wrapper:focus {
  background-color: #3e4451;
}


</style>