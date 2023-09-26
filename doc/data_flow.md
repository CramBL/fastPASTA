# Dataflow in fastPASTA
- [Dataflow in fastPASTA](#dataflow-in-fastpasta)
  - [Simple overview](#simple-overview)
  - [Detailed overview](#detailed-overview)


## Simple overview

When executing fastPASTA with in  `check`-mode e.g.
```shell
fastpasta check all its bin.raw
```
The resulting data flow is illustrated below viewed from the highest level.

Each orange box runs in a seperate thread.
Pipelines are shared between threads and are essentially thread-safe FIFOs.
![pipeline-figure](fastpasta-pipeline.excalidraw.svg)

## Detailed overview

The state diagram below offers more details to illustrate the data flow than in the [Simple overview](#simple-overview) section. For even more details you will have to look into the source code.

```mermaid
stateDiagram-v2

[*] --> ACTIVE

state ACTIVE {
    %% 1. Stage: loading the CDP from raw data

    state "1. Read CDP from raw data" as S1_READ_CDP {

        state "Read RDH" as read_rdh
        note left of read_rdh
            RDH is immediately deserialized
            and a basic sanity check is performed
        end note

        state "Read Payload" as read_payload
        note left of read_payload
            The value of the RDH offset_to_next field
            determines how many bytes are read
            and interpreted as the payload. The payload
            stays as an array of raw bytes throughout
            analysis (for performance reasons)
        end note

        state is_end_of_data <<choice>>

        [*] --> read_rdh
        read_rdh --> read_payload
        read_payload --> DATA_CHANNEL : Send CDP
        read_payload --> is_end_of_data
        is_end_of_data --> read_rdh : More data
        is_end_of_data --> [*] : EOF/End of stream
    }

    --
    %% Intermediate step (IPC)
    DATA_CHANNEL : DATA CHANNEL
    S2_DISPATCH_CDP --> DATA_CHANNEL : Get CDP
    DATA_CHANNEL --> S2_DISPATCH_CDP : Receive CDP
    note right of DATA_CHANNEL
    Simply a FIFO suitable for
    Inter-Process Communication
    end note
    --

    %% 2. Stage: Dispatching the CDP to a specific thread handling analysis
    %%            for the ID found in the RDH from the received CDP
    state "2. Dispatch CDP by ID for validation" as S2_DISPATCH_CDP {

        state "Wait for CDP" as s2_wait
        state s2_is_cdp_available <<choice>>

        [*] --> s2_is_cdp_available
        s2_is_cdp_available --> s2_wait : CDP FIFO empty
        s2_wait --> s2_is_cdp_available
        s2_is_cdp_available --> S2_SUB_DISPATCH_CDP : Got CDP
        s2_is_cdp_available --> [*] : CDP FIFO closed


        state "Dispatch CDP" as S2_SUB_DISPATCH_CDP {
            state define_id <<choice>>
            state "FEE ID" as use_fee_id
            state "GBT Link ID" as use_link_id

            state is_id_active <<choice>>

            state "Spawn validator and channel for ID" as spawn_validator
            state "Find channel matching ID" as find_channel_id

            state "Send CDP to Validator thread" as send_cdp_to_validator

            [*]  --> define_id : ID definition to use for\ninterpreting the RDH\nFEEID field
            define_id --> use_fee_id : Validating ITS Stave
            define_id --> use_link_id : Validating Other

            use_fee_id --> is_id_active
            use_link_id --> is_id_active

            note right of is_id_active
            Check a list of active threads
            for one analyzing the ID
            matching the current RDH
            end note


            is_id_active --> find_channel_id : Thread found for ID

            is_id_active --> spawn_validator : Thread not found for ID

            find_channel_id --> send_cdp_to_validator
            spawn_validator --> send_cdp_to_validator

            send_cdp_to_validator --> [*]
            send_cdp_to_validator --> VALIDATOR_THREAD_CHANNEL : Send CDP
        }
        S2_SUB_DISPATCH_CDP --> s2_is_cdp_available
    }
    --
    %% Intermediate step (IPC)
    VALIDATOR_THREAD_CHANNEL : VALIDATOR THREAD CHANNEL
    VALIDATOR_THREAD_CHANNEL --> S3_ANALYZE_CDP : Receive CDP
    S3_ANALYZE_CDP --> VALIDATOR_THREAD_CHANNEL : Get CDP
    --

    %% 3. Stage: Performing validation on the received CDP a specific ID
    state "3. Analyze CDP for a specific ID" as S3_ANALYZE_CDP {

        state "Wait for CDP" as s3_wait
        state s3_is_cdp_available <<choice>>

        [*] --> s3_is_cdp_available
        s3_is_cdp_available --> s3_wait : CDP FIFO empty
        s3_wait --> s3_is_cdp_available
        s3_is_cdp_available --> S3_SUB_DO_VALIDATE : Got CDP
        s3_is_cdp_available --> [*] : CDP FIFO closed

        state "Validate" as S3_SUB_DO_VALIDATE {
            state "RDH sanity checks" as rdh_sanity_checks
            state running_checks_enabled <<choice>>
            state "RDH running checks" as rdh_running_checks
            state is_empty_payload <<choice>>
            state "Payload checks" as payload_checks

            [*] --> rdh_sanity_checks
            rdh_sanity_checks --> running_checks_enabled : Running checks?
            running_checks_enabled --> rdh_running_checks : Enabled
            running_checks_enabled --> is_empty_payload : Disabled

            rdh_running_checks --> is_empty_payload

            is_empty_payload --> payload_checks : Has payload\nAND\npayload checks enabled
            is_empty_payload -->  [*] : No payload \nOR\npayload checks disabled

            payload_checks --> [*]
        }
        S3_SUB_DO_VALIDATE --> s3_is_cdp_available
    }
}
```
