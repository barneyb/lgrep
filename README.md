# lgrep

`lgrep` is a `grep`-like utility which better understands log files. Log records in log files often correspond to lines
of text, but not always. Since `grep` only understands lines of text, this can require some gymnastics to extract full
log records. Concretely, `app.log` is an 11-line file containing four log records (of one, eight, one, and one lines):

```
% cat app.log
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

If you're `grep`ing for errors, you might try this:

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

Part of the stack trace is there, but also the unrelated message at the end (and a `--`). What you want is exactly the
matching log records, each one in its entirety, and nothing else. That's `lgrep`:

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
full log record and then check if it matches the pattern. If it matches, the whole record is printed out. This also
means you can apply multi-line patterns! For example, a stacktrace where cookbook delegated to Spring:

```
% lgrep 'springframework.*\n.*cookbook' app.log
2024-07-01 01:25:47.755 Unexpected error occurred in scheduled task
org.springframework.transaction.CannotCreateTransactionException: Could not open JPA EntityManager for transaction
    at org.springframework.orm.jpa.JpaTransactionManager.doBegin(JpaTransactionManager.java:466)
    at org.springframework.transaction.support.AbstractPlatformTransactionManager.startTransaction(AbstractPlatformTransactionManager.java:531)
    at org.springframework.transaction.support.AbstractPlatformTransactionManager.getTransaction(AbstractPlatformTransactionManager.java:405)
    at org.springframework.transaction.support.TransactionTemplate.execute(TransactionTemplate.java:137)
    at com.brennaswitzer.cookbook.async.QueueProcessor.drainQueueInternal(QueueProcessor.java:68)
    ... many more frames ...
```

## Options

`lgrep` supports a number of options that `grep` supports, such as `-v` and `-i`. It also supports a few new ones, such
as `--start`, to skip lines in a file until some pattern matches. Use `-h` for a summary, or `--help` for gory detail.

## Log Format

If your log records don't start with a timestamp, use `--log-pattern` to override the default "start of record" pattern.
Each line of the input which matches the pattern starts a new record. If you want `lgrep` to behave like `grep`, pass
`--log-pattern=` to match every line, and therefore equate records with lines. If your application consistently formats
its logs (🤞), the `LGREP_LOG_PATTERN` environment variable can be used instead of supplying `--log-pattern` all over
the place. The option still takes precedence, for ad hoc use.

## Compressed Logs

`lgrep` transparently supports compressed inputs using compression utilities available on your `$PATH`. This means there
is process overhead, as well as decompression overhead. To demonstrate, compress `app.log` a couple ways:

```
% gzip -k app.log
% bzip2 -k app.log
% ls -o app.log*
-rw-r--r--  1 barneyb  950 Jul  6 19:29 app.log
-rw-r--r--  1 barneyb  479 Jul  6 19:29 app.log.bz2
-rw-r--r--  1 barneyb  417 Jul  6 19:29 app.log.gz
```

Whether named on the command line or streamed, `lgrep` will treat them equivalently:

```
% lgrep draining app.log
2024-07-01 01:25:46.123 draining queue
2024-07-01 01:25:47.790 queue draining complete (ERROR)
% lgrep draining app.log.gz
2024-07-01 01:25:46.123 draining queue
2024-07-01 01:25:47.790 queue draining complete (ERROR)
% lgrep draining < app.log.bz2
2024-07-01 01:25:46.123 draining queue
2024-07-01 01:25:47.790 queue draining complete (ERROR)
```
