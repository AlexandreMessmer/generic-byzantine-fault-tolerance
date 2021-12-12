# Implementation and design decisions



### 		System layout	

​	The system is a set of abstractions named peers. Each peer can communicate with any other peer, even itself. There is two main peers, the client and the replica. Each peer is either a client or a replica. These two abstractions have each a common behavior upon receiving any messages. 

`ByzantineSystem`: System that contains byzantine peers

API: 

- setup(n, m) : Create an instance of `ByzantineSystem` with n `Client` and m `Replica`. (Uses default settings)

  ***TODO***: Add settings as parameters

- send_command(cmd, id) : Sends a `Command` *cmd* to the client (or replica) with identificator *id*. This returns a `FeedbackReceiver`, which can be used to get feedback from the system (e.g. a result computed by a client).



### 	Communicate 

​	The system can send `Command` to its peers. This is a local way to simulate a distributed systems. If the system was fully distributed, each client would send its request directly to the system. However, due to the local implementation, we must restrain ourselves to a global entity that simulates these requests. This is the duty of the system.

​	Peers can communicate through `Message`s. They handle messages as defined in the paper. When a client issues a request, he needs to get a response. Here, we defined a non-blocking way of waiting for the response. Each client keep track of the response received from a request, and once he gets enough of them, he will provide a correct result.

### 2. How to simulate a transmission delay

​	First, there is a subtential need to delay the messages transmitted through the network. For some pratical reason, this delay will be simulated by the receiver. We can see it in two different ways :

- Either the sender waits a certain amount of time to send the message. This essentially shift its whole outbox.
- Or the receiver waits a certain amount of time before processing the messages. This does not modify its behavior, since he only responds to what he receives.

The more elegant way to do it is to make the receiver waits. Note that we only delay the messages. The command are essentially order given by the system to its peer. It is the way rust enables the programmer to interact with a network.



### 3. Implementation of the client



- Handling requests:

​	In a real distributed system, the client issues request (e.g. performing some computations). Due to the mock implementation, a request comes from an `Instruction`, i.e. a couple of `Command` and `FeedbackSender`. A client can execute an instruction on the whole system and return a result. To get a correct result, he needs to receive a certain amount of identical messages to be sure that the system computed it correctly. Thus, a client must keep track of the request he is performing. Each request is defined by a `RequestId`. Peers can answer a client's request with `ACK` and `CHK` messages. When the client receives one of them, he tracks how many time he receives it (i.e. how many peer sent him the message *m*). The number of messages received is tracked in a `Database` instance. 

Database: 

- received: Messages received
- Requests: Requests to handle



# Tasks



-  Refactoring message passing: Add a `FeedbackChannel` together with a `Command` [x]
-  Define the interface for the `Database` []

