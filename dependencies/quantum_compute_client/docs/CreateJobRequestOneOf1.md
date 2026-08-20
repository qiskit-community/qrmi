# CreateJobRequestOneOf1

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**program_id** | **String** | ID of the program to be executed | 
**backend** | **String** | Name that identifies the backend on which to run the program. | 
**runtime** | Option<**String**> | Name and tag of the image to use when running a program (IBM Quantum channel users only). Should follow the pattern \"name:tag\". | [optional]
**tags** | Option<**Vec<String>**> | List of job or program tags | [optional]
**log_level** | Option<**String**> | Logging level of the program | [optional]
**cost** | Option<**i32**> | Cost of the job as the estimated time it should take to complete (in seconds). Should not exceed the cost of the program | [optional]
**session_id** | Option<**String**> | Identifier of the session that the job is a part of | [optional]
**remote_storage** | [**models::JobResponseRemoteStorage**](JobResponse_remote_storage.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


