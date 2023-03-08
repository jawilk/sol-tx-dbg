<template>
  <table class="variables-table">
    <tr class="header">
      <th class="table-entry">name</th>
      <th class="table-entry2">value</th>
    </tr>
    <tr v-for="(variable, index) in variablesParsed" :key="index">
      <td>({{ variable.type }}) {{ variable.name }}</td>
      <td>
        <template v-if="variable.type === variable.value">
          variable not available
        </template>
        <template v-else>
          {{ variable.value }}
        </template>
      </td>
    </tr>
  </table>
</template>
  
<script>
import * as bs58 from "bs58";

export default {
  name: "VariablesComp",
  props: ["variables", "getMemory"],
  data() {
    return {
      pubkeyRegex: /0x[0-9a-fA-F]+: ((?:[0-9a-fA-F]{2}\s){16})/g,
      variablesParsed: [],
    };
  },
  watch: {
    async variables() {
      let variables = this.variables;

      for (let i = 0; i < variables.length; i++) {
        if (variables[i].type.includes("solana_program::pubkey::Pubkey *")) {
          const output = await this.getMemory(variables[i].value, 32, false);
          console.log("output:", output);
          let matches;
          let byteString = "";
          while ((matches = this.pubkeyRegex.exec(output)) !== null) {
            byteString += matches[1].replace(/ /g, "");
          }
          console.log("byteString:", byteString);
          const bytes = new Uint8Array(
            byteString.match(/.{1,2}/g).map((byte) => parseInt(byte, 16))
          );
          const pubkey = bs58.encode(bytes);
          console.log("pubkey:", pubkey);
          variables[i].value = pubkey + " @ " + variables[i].value;
        }
      }
      this.variablesParsed = variables;
    },
  },
};
</script>
  
  <style>
.variables-table {
  color: #e0e4e6;
  font-size: 13px;
  padding-right: 100px;
}
</style>