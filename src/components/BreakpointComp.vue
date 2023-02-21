<template>
      <!-- <div class="panel-header">
      <p class="title">breakpoints</p>
      <div class="switch-container" title="remove">
      <label class="switch">
        <input type="checkbox">
        <span class="slider round"></span>
      </label>
      </div>  
      </div> -->
      <table class="dis-table">
        <tr class="header">
          <th class="delete-breakpoint"></th>
          <th class="table-entry">line</th>
          <th class="table-entry2">file</th>
        </tr>
        <template v-if="breakpoints">
        <template v-for="(row, index) in Object.entries(breakpoints)" :key="index">
          <tr v-for="(value, i) in row[1]" :key="i">
          <td @click="deleteBreakpoint(row[0], value)" class="delete">x</td>
              <td>{{ value }}</td>
              <td>{{ row[0] }}</td>
          </tr>          
        </template>
      </template>
      </table>
</template>
  
  <script>
  export default {
    name: 'BreakpointComp',
    props: ['breakpoints'],
  //   mounted() {
  //     this.$nextTick(function () {
  //   document.querySelector('cm-breakpoint-gutter').addEventListener('mousedown', this.breakpointEvent());
  //     });
  // },
  methods: {
    deleteBreakpoint(fileName, line) {
      console.log('deleteBreakpoint', fileName, line);
      this.$emit('deleteBreakpoint', fileName, line);
    },
  }
  }
  </script>

  <style>
.header {
  text-align: left;
}

.delete {
  cursor: pointer;
  color: #e06c75;
}

.delete-breakpoint {
  padding-right: 20px;
}

.table-entry {
  padding-right: 50px;
}

.table-entry2 {
  padding-right: 190px;
}

.dis-table {
  color: #E0E4E6;
  font-size: 1.2em;
}

.yellow-background {
  background-color: #595910;
}
</style>
  