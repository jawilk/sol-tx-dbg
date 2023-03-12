<template>
  <div class="memory-input-wrapper">
    <form class="tx-hash-input" @submit.prevent="executeCommand">
      <input
        style="width: 175px"
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
      <select class="format-select" v-model="format">
        <option value="" disabled selected hidden>Select an option</option>
        <option value="no format">no format</option>
        <option value="pubkey">pubkey</option>
      </select>
      <button type="submit" style="cursor: pointer; margin-left: 10px">
        go
      </button>
    </form>
  </div>
  <pre class="memory-output">{{ output }}</pre>
</template>
 
 <script>
import * as bs58 from "bs58";

export default {
  name: "MemoryComp",
  props: ["getMemory"],
  data() {
    return {
      address: "",
      bytes: "",
      format: "no format",
      output: "",
      pubkeyRegex: /0x[0-9a-fA-F]+: ((?:[0-9a-fA-F]{2}\s){16})/g,
    };
  },
  methods: {
    async executeCommand() {
      switch (this.format) {
        case "pubkey": {
          const output = await this.getMemory(this.address, 32, true);
          let matches;
          let byteString = "";
          while ((matches = this.pubkeyRegex.exec(output)) !== null) {
            byteString += matches[1].replace(/ /g, "");
          }
          const bytes = new Uint8Array(
            byteString.match(/.{1,2}/g).map((byte) => parseInt(byte, 16))
          );
          this.output = bs58.encode(bytes);
          break;
        }
        default:
          this.output = await this.getMemory(this.address, this.bytes);
          break;
      }
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

.format-select {
  margin-left: 10px;
}
</style>
 