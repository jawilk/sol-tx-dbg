<template>
  <div class="intro">
    <p>
      This is experimental, for a quick start you can try the following tx:<br /><br />

      <b
        >oLjrBfTsbgSYu6wZBP8Yw5HAiVA7vS4NmAk21X5tuHBJ1FTo9UGW8EWzGBqXCqAGTDjTNXz9J4G8HQuV95ubEU2</b
      ><br /><br />

      Currently, the following programs are supported as the main program in an
      instruction (cpis to unknown ones will just be fast-forwarded):<br /><br />

      <b>ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL</b><br />
      <b>TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA</b><br />
      <b>CrX7kMhLC3cSsXJdT7JDgqrRVWGnUpX3gfEfxxU2NVLi</b> (Lido)<br />
      <b>srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX</b> (OpenBook DEX)<br />
      <b>PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY</b> (Phoenix V1)<br />
      <b>So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo</b> (Solend)<br />
      <br />
      <b>Please note:</b> Since we use the current on-chain state for all
      accounts, it is possible that previously successful transactions may now
      fail.<br />
      Stepping over/in can be quite cumbersome, breakpoints+continue will lead
      to a smoother experience.<br />
      <br />
      <b>Legacy transactions only for now!</b>
    </p>
  </div>
</template>
  
  <script>
export default {
  name: "IntroComp",
};
</script>
  
  <style>
.intro {
  left: 30%;
  position: absolute;
  top: 200px;
  text-align: left;
  width: 100%;
  border: none;
  background-color: transparent;
  color: white;
  font-size: 14px;
}
</style>