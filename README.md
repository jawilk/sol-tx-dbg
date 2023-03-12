[image0]: ./media/view_0.png
[image1]: ./media/view_1.png

https://sol-tx.wtf
# sol-tx-dbg
![alt text][image0]
![alt text][image1]

# Setup
Init  
```
~: git clone https://github.com/jawilk/sol-tx-dbg
~: cd sol-tx-dbg
~: git submodule update --init --recursive
~: git lfs pull
```
Build app  
```
:~ cd app
~: yarn install
~: yarn serve --port 8084
```
Build poc  
```
~: cd backend/poc
~: cargo build
```
Build server  
```
~: cd backend/server
~: cargo build
```
Install websockify  
```
~: git clone --branch sol-tx-dbg https://github.com/jawilk/websockify
~: cd websockify
~: python3 setup.py install
```
# Start  
(i) Server  
```
~: cd backend/server
~: cargo run
```
(ii) Poc  
```
~: cd backend/websockify
~: websockify :9007 -v --token-plugin PortOnly --token-source temp.txt
```
