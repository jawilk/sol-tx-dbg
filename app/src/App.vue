<template>
  <div class="tx-hash-wrap">
    <form class="tx-hash-input" @submit.prevent="loadTx">
      <input
        style="width: 400px"
        key="input"
        type="text"
        placeholder="tx hash"
        v-model="txHash"
        role="link"
      />
      <button style="cursor: pointer">go</button>
    </form>
    <span class="own-wrap">
      <button
        class="own-item"
        @click="toggleLocalExecution"
        title="start local execution"
      >
        local
      </button>
    </span>
    <span class="github-wrap">
      <button
        class="github-button"
        onclick="window.open('https://github.com/jawilk/sol-tx-dbg', '_blank')"
        title="github"
      >
        <i class="fa">&#xf09b;</i>
      </button>
    </span>
  </div>
  <localComp v-if="showLocal" :showLocal="showLocal"
  @close="toggleLocalExecution" />
  <router-view />
</template>

<script>
import LocalComp from "./components/LocalComp.vue";

export default {
  name: "App",
  components: {
    LocalComp,
  },
  data() {
    return {
      txHash: "",
      showLocal: false,
    };
  },
  methods: {
    loadTx() {
      // console.log("load tx", this.txHash);
      this.$router.push({ name: "choose", query: { txHash: this.txHash } });
    },
    toggleLocalExecution() {
      this.showLocal = !this.showLocal;
    },
  },
};
</script>

<style>
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans",
    Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji";
  font-size: 12px !important;
  background-color: #201c1c;
}
/* width and height of the scrollbar track */
::-webkit-scrollbar {
  width: 1px;
  height: 1px;
}

/* background color of the scrollbar track */
::-webkit-scrollbar-track {
  background-color: transparent;
}

/* color of the scrollbar thumb */
::-webkit-scrollbar-thumb {
  background-color: #888;
}

/* color of the scrollbar thumb on hover */
::-webkit-scrollbar-thumb:hover {
  background-color: #555;
}

.tx-hash-wrap {
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
  font-size: 13px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica,
    Arial, sans-serif;
}

.tx-hash-input {
  display: inline-block;
  margin-left: auto;
  margin-right: auto;
  text-align: left;
}

.own-wrap {
  position: absolute;
  background: #e0e4e6;
  padding: 1px;
  border-radius: 6px;
  border-width: 1em;
  right: 10%;
  width: 70px;
  height: 25px;
}

.own-item {
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

.github-wrap {
  position: absolute;
  background: transparent;
  padding: 1px;
  border-radius: 6px;
  border-width: 1em;
  right: 0%;
  width: 50px;
  height: 30px;
}

.github-button {
  position: absolute;
  background: transparent;
  border: none;
  left: 50%;
  top: 50%;
  width: 100%;
  height: 100%;
  transform: translate(-50%, -50%);
  color: #e0e4e6;
  font-size: 23px;
  cursor: pointer;
}
</style>