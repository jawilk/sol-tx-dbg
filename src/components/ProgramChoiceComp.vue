<template>
  <div class="program-choice-header">
    <p>
      This transaction contains
      <span style="color: #98c379">{{ instData.length }}</span> instructions.
      Please choose:
    </p>
  </div>
  <div class="program-view">
    <div class="program-wrap" v-for="(program, index) in instData" :key="index">
      <div class="program">
      <div class="program-header">{{ index }}</div>
      <div class="program-info">
        Program ID: {{ program.program_id }}<br /><br />
        Program Name: {{ program.name ?? "Unknown" }}<br /><br />
        CPIs: {{ program.cpi_programs ? program.cpi_programs.join(', ') : "None" }}<br /><br />
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
  </div>
</template>
  
      <script>
export default {
  name: "ProgramChoiceComp",
  data() {
    return {
      uuid: "",
      instData: [],
    };
  },
  async mounted() {
    console.log("query new tx", this.$route.query);
    this.tx_hash = this.$route.query.txHash;
    const response = await fetch("http://localhost:8000/init/" + this.tx_hash);
    const responseJson = await response.json();
    this.uuid = responseJson.uuid;
    this.instData = responseJson.program_metas;
    console.log(this.instData, this.uuid);
  },
  methods: {
    startDebugger(index) {
      console.log(
        "start debugger",
        this.instData[index].program_id + "_" + this.uuid
      );
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

.program {
  width: 400px;
  height: 600px;
  display: flex;
  background: #201c1c;
  overflow: scroll;
  border-radius: 6px;

}

.program-wrap {
  margin-right: 20px;
  position: relative;
  background: #e0e4e6;
  padding: 1px;
  border-radius: 6px;
  border-width: 1em;
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
  position: relative;
  margin: auto;
}

.startBtn {
  width: 100px;
  height: 100px;
  position: absolute;
  cursor: pointer;
  bottom: 10px;
}

.startBtnDis {
  color: #858585 !important;
  cursor: not-allowed !important;
}
</style>