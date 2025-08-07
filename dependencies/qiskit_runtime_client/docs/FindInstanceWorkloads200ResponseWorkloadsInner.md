# FindInstanceWorkloads200ResponseWorkloadsInner

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Workload id (job and session id) | 
**created** | **String** | Creation date | 
**ended** | Option<**String**> | End date | [optional]
**backend** | **String** | Backend name | 
**instance** | **String** | Instance as hub/group/project | 
**user_id** | **String** | User id | 
**accepting_jobs** | Option<**bool**> | true if the session accepts jobs, false otherwise. Only for sessions, null for jobs | [optional]
**mode** | **String** | Workload mode: job, session or batch | 
**status** | **String** | State for the workload. | 
**status_reason** | Option<**String**> | Jobs only, status reason for the job | [optional]
**tags** | Option<**Vec<String>**> | Tags for the jobs | [optional]
**usage_seconds** | Option<**f64**> | Usage in seconds. Can be null for ongoing workloads. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


