[image0]: ./media/view_0.png
[image1]: ./media/view_1.png

Step-by-step replay of a solana transaction in the browser, with CPI support. It uses an lldb wasm build in the browser and an execution environemnt with the vm on sever side (slow). If a CPI is encountered, a new browser tab is opened.
A demo can be found here https://www.youtube.com/watch?v=42N-S7gg3-Q.  
https://sol-tx.wtf (inactive)
# sol-tx-dbg
![alt text][image0]
![alt text][image1]

# Setup
Init  
```
git clone https://github.com/jawilk/sol-tx-dbg
cd sol-tx-dbg
git submodule update --init --recursive
git lfs pull
```
Build app  
```
cd app
yarn install
```
Build poc  
```
cd backend/poc
cargo build
```
Build server  
```
cd backend/server
cargo build
```
Install websockify  
```
git clone --branch sol-tx-dbg https://github.com/jawilk/websockify
cd websockify
python3 setup.py install
```
# Start  
App  
```
cd app
yarn serve --port 8084
```
Server  
```
cd backend/server
cargo run
```
Poc  
```
cd backend/websockify
websockify :9007 --token-plugin PortOnly --token-source temp.txt
```
