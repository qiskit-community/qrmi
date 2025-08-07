# CreateSession200ResponseTimestampsInner

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status** | **String** | The state of the session. - open: The session is waiting to run jobs. - active: The session has priority to run jobs on the backend and is running jobs or is waiting for more jobs to run. - inactive: The session does not have priority and is not running any jobs. - closed: The session is not running any jobs and will not accept/run new jobs.  | 
**timestamp** | **String** | Timestamp of when the session transitioned into the given status | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


