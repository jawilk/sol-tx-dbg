<template>
    <div class="dis-view">
      <div class="panel-header">
        <p class="title">disassembly</p>
        <div class="switch-container" title="remove">
        <label class="switch">
          <input type="checkbox">
          <span class="slider round"></span>
        </label>
        </div>  
        </div>
      <table class="dis-table">
        <tr class="header">
          <th class="table-entry">addr</th>
          <th class="table-entry2">inst</th>
          <th class="table-entry">mnem</th>
        </tr>
        <tr v-for="(item, index) in disData" :key="item.addr" :class="{ 'yellow-background': index === 0 }">
          <td>{{ item.addr }}</td>
          <td>{{ item.inst }}</td>
          <td>{{ item.mnem }}</td>
        </tr>
      </table>
    </div>
</template>
  
  <script>
  export default {
    name: 'DisassemblyComp',
    props: ['disData'],
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

.header {
  text-align: left;
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
  