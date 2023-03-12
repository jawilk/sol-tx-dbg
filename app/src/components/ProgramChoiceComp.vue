<template>
  <div class="program-choice-header">
    <span>
      This transaction contains
      <span style="color: #98c379">{{ instData.length }}</span> instruction{{
        pluralize
      }}. Please choose:
    </span>
  </div>
  <div class="program-view">
    <div class="program-wrap" v-for="(program, index) in instData" :key="index">
      <div class="program-header">{{ index }}</div>
      <div class="program-info">
        Program Name: {{ program.name ?? "Unknown" }}<br /><br />
        Program ID: {{ program.program_id }}<br /><br />
        CPIs:
        {{ program.cpi_programs ? program.cpi_programs.join(", ") : "None"
        }}<br /><br />
        <button
          v-if="!program.is_supported"
          type="submit"
          class="chooseBtn startBtnDis"
        >
          not supported
        </button>
        <button
          @click="startDebugger(index)"
          v-if="program.is_supported"
          type="submit"
          class="chooseBtn"
        >
          replay
        </button>
      </div>
    </div>
  </div>
</template>
  
      <script>
export default {
  name: "ProgramChoiceComp",
  data() {
    return {
      init_url: process.env.VUE_APP_TX_INFO_URL,
      uuid: "",
      instData: [],
      tx_hash: "",
    };
  },
  computed: {
    pluralize() {
      if (this.instData.length > 1) return "s";
      else return "";
    },
  },
  watch: {
    "$route.query.txHash": {
      handler: async function (tx_hash) {
        this.tx_hash = tx_hash;
        await this.load(tx_hash);
      },
      deep: true,
      immediate: true,
    },
  },
  methods: {
    async load(tx_hash) {
      const response = await fetch(this.init_url + tx_hash);
      const responseJson = await response.json();
      this.uuid = responseJson.uuid;
      this.instData = responseJson.tx_program_metas;
    },
    startDebugger(index) {
      this.$router.push({
        name: "program",
        query: {
          inst_nr: index.toString(),
          program_id: this.instData[index].program_id,
          uuid: this.uuid,
          tx_hash: this.tx_hash,
          program_name: this.instData[index].name,
        },
      });
    },
  },
};
</script>
  
  <style>
.program-choice-header {
  width: 100%;
  height: 40px;
  margin-top: 80px;
  font-size: 15px;
  color: #e0e4e6;
  text-align: center;
}

.program-view {
  margin-top: 40px;
  position: relative;
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: flex-end;
  width: 100%;
  color: #e0e4e6;
}

.program-wrap {
  width: 400px;
  height: 600px;
  display: flex;
  background: #201c1c;
  overflow: scroll;
  border-radius: 6px;
  position: relative;
  border-style: solid;
  border-color: #717171;
  padding: 1px;
  border-radius: 6px;
  border-width: 1px;
  margin-right: 20px;
}

.program-header {
  height: 20px;
  font-size: 14px;
  text-align: center;
  color: #e0e4e6;
}

.program-info {
  margin: auto;
}

.chooseBtn {
  width: 100px;
  height: 100px;
  position: absolute;
  left: 50%;
  bottom: 20px;
  -webkit-transform: translateX(-50%);
  transform: translateX(-50%);
  cursor: pointer;
}

.startBtnDis {
  color: #858585 !important;
  cursor: not-allowed !important;
}
</style>