/*
 * This code is part of Qiskit.
 *
 * Copyright (C) IBM 2026
 *
 * This code is licensed under the Apache License, Version 2.0. You may
 * obtain a copy of this license in the LICENSE.txt file in the root directory
 * of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
 *
 * Any modifications or derivative works of this code must retain this
 * copyright notice, and modified files need to carry a notice indicating
 * that they have been altered from the originals.
 */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "qrmi.h"

extern void load_dotenv();

/*
 * Usage: service [resource_name]
 *
 * resource_name - optional. If given, that resource is acquired, its
 *                 metadata and target are printed, then it is released.
 *                 If omitted, this just lists every accessible resource
 *                 assigned to the job.
 *
 * Unlike the other examples in this directory, which each construct a
 * single, specific resource directly (e.g. qrmi_resource_new(backend,
 * QRMI_RESOURCE_TYPE_IBM_QUANTUM_SYSTEM)), this example uses
 * qrmi_service_resources(), which discovers *all* of the QPU resources
 * assigned to the current job from the environment -- the same environment
 * variables a Slurm QRMI plugin would set -- and returns the ones that are
 * currently accessible.
 *
 * qrmi_service_resources() reads QRMI_JOB_QPU_RESOURCES / QRMI_JOB_QPU_TYPES
 * (falling back to the legacy SLURM_JOB_QPU_RESOURCES /
 * SLURM_JOB_QPU_TYPES), each a delimiter-separated list (delimiter: ","
 * by default, overridable via QRMI_LIST_DELIMITER). The two lists must be
 * the same length and pair up positionally, e.g.:
 *
 *   export QRMI_JOB_QPU_RESOURCES=ibm_torino,my_pasqal_qpu
 *   export QRMI_JOB_QPU_TYPES=qiskit-runtime-service,pasqal-cloud
 *
 * Each resource named above also needs its own vendor-specific environment
 * variables set (see the other examples in this directory for what those
 * are per vendor). This example assumes a `.env` file with all of the above
 * is available in the current directory.
 */
int main(int argc, char *argv[]) {

  const char *resource_name = argc >= 2 ? argv[1] : NULL;

  load_dotenv();

  QrmiQuantumResources resources = {0};
  QrmiReturnCode rc = qrmi_service_resources(&resources);
  if (rc != QRMI_RETURN_CODE_SUCCESS) {
    const char *last_error = qrmi_get_last_error();
    fprintf(stderr, "qrmi_service_resources() failed. %s\n", last_error);
    qrmi_string_free((char *)last_error);
    return EXIT_FAILURE;
  }

  if (resources.length == 0) {
    fprintf(stdout, "No accessible resources found for this job.\n");
    return EXIT_SUCCESS;
  }

  fprintf(stdout, "Accessible resources (%zu found):\n", resources.length);

  QrmiQuantumResource *selected = NULL;

  for (size_t i = 0; i < resources.length; i++) {
    char *id = NULL;
    rc = qrmi_resource_id(resources.resources[i], &id);
    if (rc != QRMI_RETURN_CODE_SUCCESS) {
      continue;
    }

    QrmiResourceType resource_type;
    rc = qrmi_resource_type(resources.resources[i], &resource_type);
    if (rc == QRMI_RETURN_CODE_SUCCESS) {
      const char *type_str = qrmi_config_resource_type_to_str(resource_type);
      fprintf(stdout, "  %-30s type=%s\n", id, type_str);
      qrmi_string_free((char *)type_str);
    }

    if (resource_name != NULL && selected == NULL &&
        strcmp(id, resource_name) == 0) {
      selected = resources.resources[i];
    }

    qrmi_string_free(id);
  }

  if (resource_name == NULL) {
    qrmi_service_resources_free(&resources);
    return EXIT_SUCCESS;
  }

  if (selected == NULL) {
    fprintf(stderr, "'%s' was not found among this job's accessible resources\n",
            resource_name);
    qrmi_service_resources_free(&resources);
    return EXIT_FAILURE;
  }

  fprintf(stdout, "\nAcquiring '%s'...\n", resource_name);
  char *acquisition_token = NULL;
  rc = qrmi_resource_acquire(selected, &acquisition_token);
  if (rc != QRMI_RETURN_CODE_SUCCESS) {
    const char *last_error = qrmi_get_last_error();
    fprintf(stderr, "qrmi_resource_acquire() failed. %s\n", last_error);
    qrmi_string_free((char *)last_error);
    qrmi_service_resources_free(&resources);
    return EXIT_FAILURE;
  }
  fprintf(stdout, "acquisition_token = %s\n", acquisition_token);

  QrmiResourceMetadata *metadata = NULL;
  rc = qrmi_resource_metadata(selected, &metadata);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    size_t num_keys = 0;
    char **metadata_keys = NULL;
    rc = qrmi_resource_metadata_keys(metadata, &num_keys, &metadata_keys);
    if (rc == QRMI_RETURN_CODE_SUCCESS) {
      for (size_t i = 0; i < num_keys; i++) {
        char *value = qrmi_resource_metadata_value(metadata, metadata_keys[i]);
        printf("metadata key=[%s], value=[%s]\n", metadata_keys[i], value);
        qrmi_string_free(value);
      }
      qrmi_string_array_free(num_keys, metadata_keys);
    }
    qrmi_resource_metadata_free(metadata);
  }

  char *target = NULL;
  rc = qrmi_resource_target(selected, &target);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    printf("target = %s\n", target);
    qrmi_string_free(target);
  }

  rc = qrmi_resource_release(selected, acquisition_token);
  fprintf(stdout, "qrmi_resource_release rc = %d\n", rc);
  qrmi_string_free(acquisition_token);

  // Individual handles inside `resources` (including `selected`, which
  // just points into the array -- it was never separately allocated) are
  // owned by the array and must not be freed on their own.
  qrmi_service_resources_free(&resources);

  return EXIT_SUCCESS;
}
