# Cron-cats  

The idea of this API is to automate the topping-up and creation of tasks through the API which allows users to set-and-forget their automated execution actions.

## Task creation

When a `RegisterTask` is called on this API it will forward the call and retrieve the hash from the reply (like what the vectis-dao integration does).
It will add this task hash to a list of tasks that are maintained for this specific OS.

## Automatic re-fill through meta-tasks

The instantiation of this API requires a small amount of initial funds. These funds will be used to register a meta-task. The meta-task will be set to query this contract. The query it calls will indicate weather there are tasks that need re-filling.  

If there are tasks that need to be filled the query will indicate that which will trigger an execution call on the API.

The API will then create a fixed amount of messages to:

1. Forward funds to the task that needs to be re-filled for OS X.
2. Forward a small amount of funds to itself to periodically re-fund the meta-task.

By creating a meta-task for each OS individually we can artificially set a limit on the amount of tasks that can be registered under one OS, which solves the problem of paged-query support.