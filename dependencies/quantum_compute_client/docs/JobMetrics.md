# JobMetrics

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**timestamps** | Option<[**models::JobMetricsTimestamps**](JobMetrics_timestamps.md)> |  | [optional]
**bss** | Option<[**models::JobMetricsBss**](JobMetrics_bss.md)> |  | [optional]
**usage** | Option<[**models::JobMetricsUsage**](JobMetrics_usage.md)> |  | [optional]
**executions** | Option<**i32**> | Number of executions during job | [optional]
**num_circuits** | Option<**i32**> | Number of circuits executed on quantum backend | [optional]
**num_qubits** | Option<**Vec<i32>**> | Number of qubits on quantum backend | [optional]
**circuit_depths** | Option<**Vec<i32>**> | An array of circuit depths | [optional]
**qiskit_version** | Option<**String**> | Qiskit version used during execution of the job | [optional]
**estimated_start_time** | Option<**String**> | UTC timestamp for when the job will start | [optional]
**estimated_completion_time** | Option<**String**> | UTC timestamp for when the job will complete | [optional]
**position_in_queue** | Option<**i32**> | Current position of job in queue (IBM Quantum channel users only) | [optional]
**position_in_provider** | Option<**i32**> | Current position of job in provider (IBM Quantum channel users only) | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


