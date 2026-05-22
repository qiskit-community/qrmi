# IqmServerJob

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**artifacts** | [**Vec<models::IqmServerJobArtifact>**](IqmServerJobArtifact.md) | List of the available artifacts for the job. | 
**compilation** | Option<[**models::IqmServerJobCompilation**](IqmServerJobCompilation.md)> |  | [optional]
**errors** | Option<[**Vec<models::IqmServerJobError>**](IqmServerJobError.md)> | If the job is rejected or failed, errors leading to the failure will be in this error list. | [optional]
**execution** | Option<[**models::IqmServerJobExecution**](IqmServerJobExecution.md)> |  | [optional]
**id** | **uuid::Uuid** |  | 
**messages** | [**Vec<models::IqmServerJobMessage>**](IqmServerJobMessage.md) | Informational messages related to the job processing. | 
**qc** | [**models::IqmServerJobQc**](IqmServerJobQc.md) |  | 
**queue_position** | Option<**i32**> | Job's position in the queue (if applicable) | [optional]
**runtime_ms** | Option<**i64**> |  | [optional]
**status** | [**models::IqmServerJobStatus**](IqmServerJobStatus.md) |  | 
**tag** | Option<**String**> | Optional user-defined tag associated with the job. | [optional]
**timeline** | [**Vec<models::IqmServerJobTimelineEvent>**](IqmServerJobTimelineEvent.md) | The timeline of events occurred to the job. | 
**r#type** | **String** | Type of the job | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


