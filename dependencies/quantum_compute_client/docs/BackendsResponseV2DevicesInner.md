# BackendsResponseV2DevicesInner

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The name of the backend device | 
**status** | [**models::BackendsResponseV2DevicesInnerStatus**](BackendsResponseV2_devices_inner_status.md) |  | 
**is_simulator** | Option<**bool**> | Indicates if the device is a simulator | [optional]
**qubits** | Option<**i32**> | The number of qubits in the device | [optional]
**clops** | Option<[**models::BackendsResponseV2DevicesInnerClops**](BackendsResponseV2_devices_inner_clops.md)> |  | [optional]
**processor_type** | Option<[**models::BackendsResponseV2DevicesInnerProcessorType**](BackendsResponseV2_devices_inner_processor_type.md)> |  | [optional]
**queue_length** | **i32** | The number of jobs waiting to be executed | 
**performance_metrics** | Option<[**models::BackendsResponseV2DevicesInnerPerformanceMetrics**](BackendsResponseV2_devices_inner_performance_metrics.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


