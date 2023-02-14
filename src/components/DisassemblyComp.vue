<template>
    <div class="dis-view">
      <!-- <ul class="dis-list">
        <li class="dis-line" v-for="(item, index) in disData" :key="item.id" :class="{ 'yellow-background': index === 0 }">
          {{ item.text }}
        </li>
      </ul>  -->
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
  background: #282c34
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
  