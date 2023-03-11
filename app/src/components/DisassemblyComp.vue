<template>
      <table class="dis-table">
        <tr class="header">
          <th class="table-entry">addr</th>
          <th class="table-entry2">inst</th>
          <th class="table-entry">mnem</th>
        </tr>
        <tr v-for="item in disData" :key="item.addr" :class="{ 'dis-background': item.addr.includes('->')}">
          <td>{{ item.addr }}</td>
          <td>{{ item.inst }}</td>
          <td>{{ item.mnem }}</td>
        </tr>
      </table>
</template>
  
  <script>
  export default {
    name: 'DisassemblyComp',
    props: ['disData'],
  }
  </script>

  <style>
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
  font-size: 13px;
}

.dis-background {
  background-color: #595910 !important;
}
</style>
  