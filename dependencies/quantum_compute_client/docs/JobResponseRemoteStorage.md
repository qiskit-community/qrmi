# JobResponseRemoteStorage

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | Option<**String**> |  | [optional][default to IbmcloudCos]
**region** | **String** | Region, Cross-Region, or Single Data Center as defined by IBM Cloud Object Storage (https://cloud.ibm.com/docs/cloud-object-storage?topic=cloud-object-storage-endpoints) | 
**region_type** | **String** |  | 
**bucket_crn** | **String** | Fully specified <a href=\"https://cloud.ibm.com/docs/account?topic=account-crn\">CRN</a> for the target Cloud Object Storage bucket | 
**object_name** | **String** | Name/ID of the object in the IBM Cloud Object Storage bucket. May *not* be specified in the default remote storage option. | 
**job_params** | [**models::RemoteStorageJobParams**](RemoteStorage_job_params.md) |  | 
**results** | [**models::RemoteStorageResults**](RemoteStorage_results.md) |  | 
**logs** | Option<[**models::RemoteStorageLogs**](RemoteStorage_logs.md)> |  | [optional]
**transpiled_circuits** | Option<[**models::RemoteStorageTranspiledCircuits**](RemoteStorage_transpiled_circuits.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


