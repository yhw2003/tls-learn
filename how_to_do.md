1. 生成ca私钥
```bash
openssl genrsa -out certs/ca.key 4096
```

2. ca为自己签名
```bash
# 生成签名请求csr
openssl req -new -key certs/ca.key -out certs/ca.csr
# 签名csr获得ca签名crt
openssl x509 -req -days 365 -in certs/ca.csr -signkey certs/ca.key -out ca.crt
```
   也可以直接
```bash
openssl req -new -x509 -key certs/ca.key -out certs/ca.crt -days 365
```
    
3. 生成私钥
```bash
openssl genrsa -out certs/server.key 2048
# 生成私钥csr
openssl req -new -key certs/server.key -out certs/server.csr
```
   
4. 使用ca为私钥csr签名
```bash
#必须指定-extfile来生成x509 v3的签名，rustls不支持老格式
openssl x509 -req -extfile certs/v3.ext -in certs/server.csr -CA certs/ca.crt -CAkey certs/ca.key -CAcreateserial -out certs/server.crt -days 365 -sha256
```