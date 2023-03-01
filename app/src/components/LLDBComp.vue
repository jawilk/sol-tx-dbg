<template>
     <div class="lldb-input-wrapper">
        <form class="tx-hash-input" @submit.prevent="executeCommand">
            <input
              style="width: 400px"
              key="input"
              type="text"
              placeholder="command"
              v-model="command"
            />
            <button type="submit" style="cursor: pointer">go</button>
          </form>
     </div>
     <div class="output">
        <b>Output:</b><br>
        <pre class="memory-output">{{ output }}</pre>
      </div>
</template>
  
  <script>
  export default {
    name: 'LLDBComp',
    props: ['executeLLDBCommand'],
    data() {
        return {
            command: '',
            output: '',
        };
    },
  methods: {
    async executeCommand() {
      this.output = await this.executeLLDBCommand(this.command);
      console.log(this.output);
    },
  }
  }
  </script>

  <style>
  .output {
    margin-top: 20px;
    color: #E0E4E6;
    font-size: 13px;
  }
</style>
  