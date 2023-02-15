<template>
    <div class="dis-view">
      <!-- <ul class="dis-list">
        <li class="dis-line" v-for="(item, index) in disData" :key="item.id" :class="{ 'yellow-background': index === 0 }">
          {{ item.text }}
        </li>
      </ul>  -->
      <div class="panel-header">
      <p class="title">breakpoints</p>
      <div class="switch-container" title="remove">
      <label class="switch">
        <input type="checkbox">
        <span class="slider round"></span>
      </label>
      </div>  
      </div>
      <table class="dis-table">
        <tr class="header">
          <th class="delete-breakpoint"></th>
          <th class="table-entry">line</th>
          <th class="table-entry2">file</th>
        </tr>
        <template v-for="(row, index) in Object.entries(breakpoints)" :key="index">
          <tr v-for="(value, i) in row[1]" :key="i">
          <td @click="deleteBreakpoint(row[0], value)" class="delete">x</td>
              <td>{{ value }}</td>
              <td>{{ row[0] }}</td>
          </tr>          
        </template>
      </table>
    </div>
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
  .dis-view {
    height:100%;
    width:100%;
    overflow: scroll;
    background: #201c1c;
    border-color: #30363d;
    border-style: solid;
    border-radius: 6px;
    border-width: 1px;
  }

.panel-header {
  display: flex;
  align-items: center;
  flex-direction: row;
}

.title {
  margin-top: 0;
  color: #E0E4E6;
  font-weight: bold;
  font-size: 1.2em;
  width: 50px;
}

.switch-container {
  margin-top: -8px;
  margin-left: auto;
  margin-right: 10px; 
}

.switch {
  position: relative;
  display: inline-block;
  width: 25px;
  height: 14px;
}

.switch input { 
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #98c379;
  -webkit-transition: .4s;
  transition: .4s;
}

.slider:before {
  position: absolute;
  content: "";
  height: 9px;
  width: 9px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  -webkit-transition: .4s;
  transition: .4s;
}

input:checked + .slider {
  background-color: #e06c75;
}

input:checked + .slider:before {
  -webkit-transform: translateX(10px);
  -ms-transform: translateX(10px);
  transform: translateX(10px);
}

.slider.round {
  border-radius: 24px;
}

.slider.round:before {
  border-radius: 50%;
}

.header {
  text-align: left;
}

.delete {
  cursor: pointer;
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
  