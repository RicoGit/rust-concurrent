
### Udp server that receives and executes commands.


#### For running execute in the root folder of this project:

     cargo run -p rust-tokio
     
 or
     
     cargo run -p rust-tokio -- 127.0.0.1:8088
     
Send upd packet to this server

    echo -n "test msg" | nc -4u -w1 localhost 8088
