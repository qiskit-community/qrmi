# CreateSessionRequestOneOf1

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**interactive_ttl** | **i32** | The maximum time (in seconds) between jobs to keep the session active. Must be less than or equal to `active_ttl`. | 
**active_ttl** | Option<**i32**> | The remaining time (in seconds) for the session to be in the active state while jobs are running. Must be less than or equal to `max_ttl`. Defaults to `max_ttl`. | [optional]
**max_ttl** | Option<**i32**> | The maximum time (in seconds) for the session to run, subject to plan limits. | [optional][default to 28800]
**mode** | **String** | Execution mode to run the session in | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


