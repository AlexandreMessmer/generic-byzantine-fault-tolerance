# Implementation and design decisions



### 		System layout	

​	The system is a set of abstractions named peers. Each peer can communicate with any other peer, even itself. There is two main peers, the client and the replica. Each peer is either a client or a replica. These two abstractions have each a common behavior upon receiving any messages. 

### 	Communicate 

​	The system can send `Command` to its peers. This is a local way to simulate a distributed systems. If the system was fully distributed, each client would send its request directly to the system. However, due to the local implementation, we must restrain ourselves to a global entity that simulates these requests. This is the duty of the system.

​	Peers can communicate through `Message`s. They handle messages as defined in the paper. When a client issues a request, he needs to get a response. Here, we defined a non-blocking way of waiting for the response. Each client keep track of the response received from a request, and once he gets enough of them, he will provide a correct result.

### 2. How to simulate a transmission delay

​	First, there is a subtential need to delay the messages transmitted through the network. For some pratical reason, this delay will be simulated by the receiver. We can see it in two different ways :

- Either the sender waits a certain amount of time to send the message. This essentially shift its whole outbox.
- Or the receiver waits a certain amount of time before processing the messages. This does not modify its behavior, since he only responds to what he receives.

The more elegant way to do it is to make the receiver waits. Note that we only delay the messages. The command are essentially order given by the system to its peer. It is the way rust enables the programmer to interact with a network.



### 2. Implementation of the client







# Tasks



-  Refactoring message passing: Add a `FeedbackChannel` together with a `Command` 