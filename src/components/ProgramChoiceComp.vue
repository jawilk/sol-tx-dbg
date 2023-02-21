<template>
    <div class="program-choice-header">
        <p>This transaction contains <span style="color:#98c379">{{ instData.length }}</span> main program invocations. Please choose:</p>
        </div>  
    <div class="program-view">
        <div class="program" v-for="(program, index) in instData" :key="index">
            <div class="program-info">
                Program ID: {{program.program_id}}<br><br>
                Program Name: {{program.program_info ? program.program_info.name : "Unknown"}}<br><br>
                is_supported: {{program.is_supported}}<br><br>
            </div>
            <button v-if="!program.is_supported" type="submit" class="startBtnDis">Not supported</button> 
            <button @click="startDebugger(index)" v-if="program.is_supported" type="submit" class="startBtn">Start</button> 
        </div>
    </div>
      </template>
  
      <script>
  
      export default {
        name: 'ProgramChoiceComp',
        data() {
          return {
            txHash: '',
            instData: [],
          };
        },
        async mounted() {
          this.txHash = this.$route.query.txHash;
          console.log("query new tx",this.$route.query);
          const response = await fetch("http://localhost:8000/init/" + this.txHash);
          const instData = await response.json();
          console.log(instData);
          this.instData = instData;
        },
        methods: {
          startDebugger(index) {
            console.log("start debugger", this.txHash+'_'+this.instData[index].program_id);
            this.$router.push({name: "program", query: {tx_hash: this.txHash, inst_nr: index.toString(), program_id: this.instData[index].program_id}});
          },
        }
      };
      </script>
  
  <style>
    .program-choice-header {
        width: 100%;
        height: 20px;
        margin-top: 80px;
        margin-bottom: 20px;
        font-size: 20px;
        text-align: center;
        color: #E0E4E6;
    }

  .program-view {
    margin-top: 40px;
    display: flex;
    flex-direction: row;
    justify-content: center;
    color: #E0E4E6;
  }

  .program {
    position: relative;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    margin-right: 50px;
    width: 400px;
    height: 600px;
    background: #201c1c;
    border-color: #30363d;
    border-style: solid;
    border-radius: 6px;
    border-width: 1px;
    overflow: scroll;
  }

  .program-info {
    position: absolute;
    top: 10px;
    left: 10px;
    font-size: 20px;
  }

  .startBtn  {
    width: 100px;
    height: 100px;
    position: absolute;
    cursor: pointer;
    bottom: 10px;
  }

  .startBtnDis {
    width: 100px;
    height: 100px;
    position: absolute;
    bottom: 10px;
    color: #858585 !important;
    cursor: not-allowed !important;
  }
  </style>