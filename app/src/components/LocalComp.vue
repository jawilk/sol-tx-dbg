<template>
    <div class="popup">
      <div class="popup-content">
        For a quick how-to please see the github.<br><br>
        <form @submit.prevent="submitForm">
            <div class="form-group">
                <label for="input1">execution env:</label>
              <input id="input1" type="text" v-model="input1" placeholder="localhost:9000"/>
            </div>
            <div class="form-group">
                <label for="input2">files:</label>
              <input id="input2" type="text" v-model="input2" placeholder="localhost:9001/static/"/>
            </div>
            <div class="form-buttons">
                <button type="submit" style="float:left" class="deactivated">coming soon</button>
                <button @click="closePopup" style="float:right">close</button>
            </div>
          </form>
      </div>
    </div>
  </template>
  
  <script>
  export default {
    name: "LocalComp",
    props: {
      showLocal: Boolean,
    },
    methods: {
      closePopup() {
        this.$emit("close");
      },
    },
  };
  </script>
  
  <style>
  .popup {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    right: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 15;
  }
  
  .popup-content {
    background-color: #fff;
    padding: 20px;
    border-radius: 5px;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
    max-width: 80%;
    max-height: 80%;
    overflow: auto;
  }

  .form-container {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
  }

  .form-group {
    display: flex;
    align-items: center;
    margin-bottom: 10px;
    flex-basis: calc(33.33% - 10px);
  }
  
  .form-group label {
    margin-right: 10px;
  }
  
  .form-buttons {
    margin-top: 10px;
  }
  </style>