export function parseConfig() {
    fetch(
        'http://localhost:8001/test.json'
      ).then(response => response.json())
        .then(res => {
            console.log(res.config)
       return res.config.roots;
        });
}

export function parseNodes() {
    fetch(
        'http://localhost:8001/test.json'
      ).then(response => response.json())
        .then(res => {
            console.log(res.nodes)

       return res.nodes;
        });
 }