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
        {{ lldbOutput }}
      </div>
</template>
  
  <script>
  export default {
    name: 'LLDBComp',
    props: ['lldbOutput'],
    data() {
        return {
            command: '',
        };
    },
  methods: {
    executeCommand() {
      this.$emit('executeCommand', this.command);
    },
  }
  }
  </script>

  <style>
  .output {
    margin-top: 20px;
    color: #E0E4E6;
  }
</style>
  