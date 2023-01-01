
### 关于HardState:
```
message HardState {
    uint64 term = 1;
    uint64 vote = 2;
    uint64 commit = 3;
}
```
HardState需要被持久化
- term: 当前raft节点的任期
- vote: 当前raft节点在本任期内投票给了谁(用于防止一个任期一个节点多次投票)
- commit: 当前raft节点已经commit的最高的日志索引


raft论文中commitIndex和applyIndex都是不需要持久化的,原因是其假设的stateMachine是volatile的，
但实际工程化中会选择进行持久化来提高节点的启动速度。
那为什么这里只持久化了commitIndex，而没有持久化applyIndex呢?
是否是从上次snapshot后的raftlog开始replay到commitIndex?


### 关于ConfState:
```
message ConfState {
    repeated uint64 nodes = 1;
    repeated uint64 learners = 2;
}
```

ConfState也需要被初始化，节点重启需要获取这些信息
- nodes: raft中的所有节点
- learners: raft中的learner节点(可选)
  

### 关于MsgType的描述说明：
MsgHup ：
MsgBeat ：
MsgPropose = 2;
MsgAppend = 3;
MsgAppendResponse = 4;
MsgRequestVote ：candicate发起的投票请求
MsgRequestVoteResponse ：投票请求响应结果
MsgSnapshot = 7;
MsgHeartbeat ：Leader给Follower发送的心跳请求
MsgHeartbeatResponse ：心跳响应结果
MsgUnreachable = 10;
MsgSnapStatus = 11;
MsgCheckQuorum = 12;
MsgTransferLeader = 13;
MsgTimeoutNow = 14;
MsgReadIndex : Follower向Leader发起的ReadIndex请求
MsgReadIndexResp  ：ReadIndex响应
MsgRequestPreVote ：预投票请求
MsgRequestPreVoteResponse ： 预投票响应