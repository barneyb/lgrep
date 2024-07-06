# lgrep

`lgrep` is a `grep`-like utility which better understands log files. Log records in log files often correspond to lines
of text, but not always. Since `grep` only understands lines of text, this can require some gymnastics to extract full
log records. To make it concrete, this snippet of log file contains 11 lines of text, but only four log records (of
one, eight, one, and one lines).

```
2024-07-01 01:25:46.123 draining queue
2024-07-01 01:25:47.755 Unexpected error occurred in scheduled task
org.springframework.transaction.CannotCreateTransactionException: Could not open JPA EntityManager for transaction
    at org.springframework.orm.jpa.JpaTransactionManager.doBegin(JpaTransactionManager.java:466)
    at org.springframework.transaction.support.AbstractPlatformTransactionManager.startTransaction(AbstractPlatformTransactionManager.java:531)
    at org.springframework.transaction.support.AbstractPlatformTransactionManager.getTransaction(AbstractPlatformTransactionManager.java:405)
    at org.springframework.transaction.support.TransactionTemplate.execute(TransactionTemplate.java:137)
    at com.brennaswitzer.cookbook.async.QueueProcessor.drainQueueInternal(QueueProcessor.java:68)
    ... many more frames ...
2024-07-01 01:25:47.790 queue draining complete (ERROR)
2024-07-01 01:25:48.000 some other unrelated log message
```

If we're `grep`ing for errors, we might try this:

```
% grep -i error app.log
2024-07-01 01:25:47.755 Unexpected error occurred in scheduled task
2024-07-01 01:25:47.790 queue draining complete (ERROR)
```

Hmm... Right records, but not very helpful. How about `-A`:

```
% grep -i -A 5 error app.log
2024-07-01 01:25:47.755 Unexpected error occurred in scheduled task
org.springframework.transaction.CannotCreateTransactionException: Could not open JPA EntityManager for transaction
    at org.springframework.orm.jpa.JpaTransactionManager.doBegin(JpaTransactionManager.java:466)
    at org.springframework.transaction.support.AbstractPlatformTransactionManager.startTransaction(AbstractPlatformTransactionManager.java:531)
    at org.springframework.transaction.support.AbstractPlatformTransactionManager.getTransaction(AbstractPlatformTransactionManager.java:405)
    at org.springframework.transaction.support.TransactionTemplate.execute(TransactionTemplate.java:137)
--
2024-07-01 01:25:47.790 queue draining complete (ERROR)
2024-07-01 01:25:48.000 some other unrelated log message
```

This time we got part of the stack trace, but also the unrelated message at the end (and a `--`). What we want is
exactly the matching log records, and each one in its entirety. That's `lgrep`:

```
% lgrep -i error app.log
2024-07-01 01:25:47.755 Unexpected error occurred in scheduled task
org.springframework.transaction.CannotCreateTransactionException: Could not open JPA EntityManager for transaction
    at org.springframework.orm.jpa.JpaTransactionManager.doBegin(JpaTransactionManager.java:466)
    at org.springframework.transaction.support.AbstractPlatformTransactionManager.startTransaction(AbstractPlatformTransactionManager.java:531)
    at org.springframework.transaction.support.AbstractPlatformTransactionManager.getTransaction(AbstractPlatformTransactionManager.java:405)
    at org.springframework.transaction.support.TransactionTemplate.execute(TransactionTemplate.java:137)
    at com.brennaswitzer.cookbook.async.QueueProcessor.drainQueueInternal(QueueProcessor.java:68)
    ... many more frames ...
2024-07-01 01:25:47.790 queue draining complete (ERROR)
```

The trick is that log records are clearly identifiable by "starts with a timestamp". `lgrep` uses this to build up a
full log record and then check if it matches the pattern. If it does, print it out, just like `grep`.

If your log files are structured differently, use `--log-pattern` to override the default "start of record" pattern.
Each line of the input which matches the pattern starts a new record. If you want `lgrep` to behave like `grep`, pass
`--log-pattern=` to match every line, and therefore equate records with lines.
