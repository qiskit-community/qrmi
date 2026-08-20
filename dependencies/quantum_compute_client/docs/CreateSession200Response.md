# CreateSession200Response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Job ID | 
**backend_name** | **String** | Backend name | 
**started_at** | Option<**String**> | Timestamp of when the session was started | [optional]
**activated_at** | Option<**String**> | Timestamp of when the session state was changed to active | [optional]
**closed_at** | Option<**String**> | Timestamp of when the session was closed | [optional]
**last_job_started** | Option<**String**> | Timestamp of when the last job in the session started | [optional]
**last_job_completed** | Option<**String**> | Timestamp of when the last job in the session completed | [optional]
**interactive_ttl** | Option<**i32**> | The maximum time (in seconds) between jobs to keep the session active | [optional]
**max_ttl** | Option<**i32**> | The maximum time (in seconds) for session to run, subject to plan limits | [optional]
**active_ttl** | Option<**i32**> | The remaining time (in seconds) for the session to be in the active state while jobs are running. | [optional]
**state** | Option<**String**> | The state of the session. - open: The session is waiting to run jobs. - active: The session has priority to run jobs on the backend and is running jobs or is waiting for more jobs to run. - inactive: The session does not have priority and is not running any jobs. - closed: The session is not running any jobs and will not accept/run new jobs.  | [optional]
**state_reason** | Option<**String**> | The reason for the state change. | [optional]
**accepting_jobs** | Option<**bool**> | If true, the session is actively accepting new jobs to be queued. If false, jobs will be rejected on create and the session will be immediately closed when there are no more jobs to run in the session. | [optional]
**mode** | **String** | Execution mode to run the session in | 
**timestamps** | Option<[**Vec<models::CreateSession200ResponseTimestampsInner>**](create_session_200_response_timestamps_inner.md)> |  | [optional]
**user_id** | Option<**String**> | The id of the user who created the session. | [optional]
**elapsed_time** | Option<**f64**> | Usage in seconds. Can be null for ongoing sessions. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


