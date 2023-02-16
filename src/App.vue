<template>
  <h3>HELOOOOO</h3>
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
    </div>
    <router-view />
    </template>

    <script>

    export default {
      name: 'App',
      data() {
        return {
          txHash: '',
          instData: [],
        };
      },
      methods: {
        loadTx() {
          this.$router.push({name: "choose", query: {txHash: this.txHash}});
        },
      },
    };
    </script>

<style>
body {
  font-family: -apple-system,BlinkMacSystemFont,"Segoe UI","Noto Sans",Helvetica,Arial,sans-serif,"Apple Color Emoji","Segoe UI Emoji";
  font-size: 14px;
  background-color: #201c1c;
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
  background-color: #201c1c;
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
</style>