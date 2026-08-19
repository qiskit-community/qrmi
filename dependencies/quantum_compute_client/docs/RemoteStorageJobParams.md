# RemoteStorageJobParams

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | Option<**String**> |  | [optional][default to IbmcloudCos]
**region** | **String** | Region, Cross-Region, or Single Data Center as defined by IBM Cloud Object Storage (https://cloud.ibm.com/docs/cloud-object-storage?topic=cloud-object-storage-endpoints) | 
**region_type** | **String** |  | 
**bucket_crn** | **String** | Fully specified <a href=\"https://cloud.ibm.com/docs/account?topic=account-crn\">CRN</a> for the target Cloud Object Storage bucket | 
**object_name** | **String** | Name/ID of the object in the IBM Cloud Object Storage bucket. May *not* be specified in the default remote storage option. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


