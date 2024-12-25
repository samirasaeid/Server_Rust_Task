# Solution
The implementation got into two paths:
First I started by learning Rust and understand the problem, the I got that the best way to implement the server is by using tokio carte or async-std carte which provide an async. connection to the server. I searched for the difference between them. I discovered that tokio is tailored for TCP server and has alot of features to facilate implementaion process so I decided to use it.
1. **Path 1: Using a tokio Crate**
-------------------------------------
 - First, I tried to use the tokio to provide async /await support. However, I implemnted the full code, but every time I run the tests, the function of accepting the client timeout. I debuged it for many time and I changed a lot code feature adding a time out, search for alternatives of the accept function by using importing but the resault was unsuccessful. I tried also to depug in the setting of my device by use ping, telnet and checking the firewall and the network configuration but the problem was not solved. So I tried to move for the nest path to get an alternative for Tokio.

2. **Path 2: Using a async/await with async-std**
-----------------------------------------------
I switched to aync-std with my previous code and results were positive so I continued to improve the code.

**Client Structure**
-----------
1. I added mutex feature for the stream to prevent race condition.
2. I added buffer as structure member to store the data received for each client.
3. I modified the handle function to check is the comming is a message or a request.
4. The recieved message will be encoded and sent back. if the receiced message failed; the server will transmit a error massage.
5. I added loop for handling multiple messages.

**Server Structure**
-----------------------
1. I add two additional parameters, shutdown_notidy which used to ensure a gratful shutdown after disconnecting between the server and the client and closing the socket.

2. I added the port parameter, to allow each client to connect to the server on a different port to prevent the server from crashing.

3. In run function, it is designed to handle the client async. using task::spawn function which is used to spawn a new task for each client.
4. while the flag of is_running is true, the server will continue to listen for new clients.

5. In the handle_client function. new client is initialized, and each client invoke the handle function to handle the message it sends.

6. In stop function, the server will stop listening for new clients and close all the sockets.

7. In get_port, the server will return the port number.

**client_test.rs**
--------------------
1. #ignored is removed.
2. get_random_port function is implement whcih return a random port number. 
3. Server initaialized with the port got from get_random_port function.
4. In each test case, I modified the port to match the port of the server to ensure the connectivity between the client and server.
5. async_std flag added over each test case to notify that the test is async.


Here you can document all bugs and design flaws.