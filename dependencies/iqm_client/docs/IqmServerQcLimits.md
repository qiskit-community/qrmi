# IqmServerQcLimits

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**max_circuits_per_job** | Option<**i32**> | Maximum number of circuits in a job. This field may be removed in the future. | [optional]
**max_executions_per_job** | **i64** | Maximum number of executions in a job, calculated as circuits \\* shots-per-circuit. | 
**max_instructions_per_circuit** | **i32** | Maximum number of instructions within any circuit, if applicable to the job type. | 
**max_shots_per_circuit** | Option<**i32**> | Maximum number of shots per circuit in a job. This field may be removed in the future. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


