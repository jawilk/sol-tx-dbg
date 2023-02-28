<template>
  <div class="lldb-input-wrapper">
    <form class="tx-hash-input" @submit.prevent="executeCommand">
      <input
        style="width: 400px"
        key="input"
        type="text"
        placeholder="address"
        v-model="address"
      />
      <input
        style="width: 60px; margin-left: 10px"
        key="input"
        type="text"
        placeholder="#bytes"
        v-model="bytes"
      />
      <button type="submit" style="cursor: pointer; margin-left: 10px">
        go
      </button>
    </form>
  </div>
  <pre class="memory-output">{{ output }}</pre>
</template>
 
 <script>
export default {
  name: "MemoryComp",
  props: ["getMemory"],
  data() {
    return {
      address: "",
      bytes: "",
      output: "",
    };
  },
  methods: {
    async executeCommand() {
        console.log("execute command", this.address, this.bytes);
      this.output = await this.getMemory(this.address, this.bytes);
      console.log("fdQ",this.output);
    },
  },
};
</script>

 <style>
.memory-output {
  margin-top: 20px;
  color: #e0e4e6;
  font-size: 13px;
  width: 100%;
  height: 100%;
}
</style>
 