# JobResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Identifier assigned to the job | 
**backend** | Option<**String**> | &ast;&ast;Warning:&ast;&ast; While this parameter is not currently required for requests, specifying it is strongly encouraged. Running an ISA circuit on a backend that has a different instruction set will result in an error. The backend parameter will be required in a future release.  The backend on which to run the program.  If no backend is specified, the job is sent to the backend with the shortest queue that you have access to.  | [optional]
**state** | [**models::JobState**](JobState.md) |  | 
**status** | **String** | Current status of the job | 
**params** | Option<[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)> | Parameters used to execute the job | [optional]
**program** | [**models::JobResponseProgram**](JobResponse_program.md) |  | 
**created** | **String** | UTC timestamp for when the job was created | 
**runtime** | Option<**String**> | Name and tag of the image to use when running a program (IBM Quantum channel users only) | [optional]
**cost** | **i32** | Cost of the job | 
**tags** | Option<**Vec<String>**> | List of job or program tags | [optional]
**remote_storage** | Option<[**models::JobResponseRemoteStorage**](JobResponse_remote_storage.md)> |  | [optional]
**session_id** | Option<**String**> | Identifier of the session that the job is a part of | [optional]
**user_id** | Option<**String**> | The id of the user submitted the job | [optional]
**usage** | Option<[**models::Usage**](Usage.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


