/*
 * (C) Copyright IBM 2026.
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
#include "common.h"

int main(int argc, char *argv[]) {

  if (argc != 3) {
    fprintf(stderr, "resource_status <resource_type> <resource_id>\n");
    return EXIT_SUCCESS;
  }

  load_dotenv();

  QrmiResourceType resource_type;

  if (!strcmp(argv[1], "ibm-quantum-system")) {
    resource_type = QRMI_RESOURCE_TYPE_IBM_QUANTUM_SYSTEM;
  } else if (!strcmp(argv[1], "qiskit-runtime-service")) {
    resource_type = QRMI_RESOURCE_TYPE_QISKIT_RUNTIME_SERVICE;
  } else if (!strcmp(argv[1], "ibm-quantum-compute")) {
    resource_type = QRMI_RESOURCE_TYPE_IBM_QUANTUM_COMPUTE_SERVICE;
  } else if (!strcmp(argv[1], "pasqal-cloud")) {
    resource_type = QRMI_RESOURCE_TYPE_PASQAL_CLOUD;
  } else if (!strcmp(argv[1], "pasqal-local")) {
    resource_type = QRMI_RESOURCE_TYPE_PASQAL_LOCAL;
  } else if (!strcmp(argv[1], "alice-bob-felis")) {
    resource_type = QRMI_RESOURCE_TYPE_ALICE_BOB_FELIS;
  } else if (!strcmp(argv[1], "iqm-server")) {
    resource_type = QRMI_RESOURCE_TYPE_IQM_SERVER;
  } else {
    fprintf(stderr, "resource type: %s is not supported.", argv[1]);
    return EXIT_FAILURE;
  }

  QrmiQuantumResource *qrmi = qrmi_resource_new(argv[2], resource_type);
  if (!qrmi) {
    const char *last_error = qrmi_get_last_error();
    fprintf(stderr, "Failed to create QRMI for %s/%s. %s (%d)\n", argv[2],
            resource_type, last_error, qrmi_get_last_error_kind());
    qrmi_string_free((char *)last_error);
    return EXIT_FAILURE;
  }

  QrmiReturnCode rc = QRMI_RETURN_CODE_SUCCESS;
  QrmiResourceStatus *status = NULL;
  rc = qrmi_resource_status(qrmi, &status);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    QrmiResourceStatusCode code;
    rc = qrmi_resource_status_code(status, &code);
    if (rc == QRMI_RETURN_CODE_SUCCESS) {
      switch (code) {
      case QRMI_RESOURCE_STATUS_CODE_ONLINE:
        fprintf(stdout, "online\n");
        break;
      case QRMI_RESOURCE_STATUS_CODE_OFFLINE:
        fprintf(stdout, "offline\n");
        break;
      case QRMI_RESOURCE_STATUS_CODE_PAUSED:
        fprintf(stdout, "paused\n");
        break;
      case QRMI_RESOURCE_STATUS_CODE_BUSY:
        fprintf(stdout, "busy\n");
        break;
      }
    }
    fprintf(stdout, "%s\n", qrmi_resource_status_code_to_string(code));

    char *reason = qrmi_resource_status_reason(status);
    if (reason != NULL) {
      fprintf(stdout, "reason=%s\n", reason);
      qrmi_string_free(reason);
    } else {
      fprintf(stdout, "no status reason reported\n");
    }
  } else {
    const char *last_error = qrmi_get_last_error();
    fprintf(stderr, "qrmi_resource_is_accessible() failed. %s (%d)\n",
            last_error, qrmi_get_last_error_kind());
    qrmi_string_free((char *)last_error);
    goto error;
  }

  QrmiResourceCapacity *capacity = NULL;
  rc = qrmi_resource_status_capacity(status, &capacity);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    fprintf(stdout, "available_slots=%u\n", capacity->available_slots);
    qrmi_resource_capacity_free(capacity);
  } else {
    const char *last_error = qrmi_get_last_error();
    fprintf(stderr, "qrmi_resource_status_capacity() failed. %s (%d)\n",
            last_error, qrmi_get_last_error_kind());
    qrmi_string_free((char *)last_error);
  }

  bool accessible = false;
  rc = qrmi_resource_status_is_accessible(status, &accessible);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    fprintf(stdout, "accessible=%d\n", accessible);
  } else {
    const char *last_error = qrmi_get_last_error();
    fprintf(stderr, "qrmi_resource_status_is_accessible() failed. %s (%d)\n",
            last_error, qrmi_get_last_error_kind());
    qrmi_string_free((char *)last_error);
  }

  bool healthy = false;
  rc = qrmi_resource_status_healthy(status, &healthy);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    if (healthy == true) {
      fprintf(stdout, "system is healthy\n");
    } else {
      fprintf(stdout, "system is unhealthy\n");
    }
  } else {
    const char *last_error = qrmi_get_last_error();
    fprintf(stderr, "qrmi_resource_status_healty() failed. %s (%d)\n",
            last_error, qrmi_get_last_error_kind());
    qrmi_string_free((char *)last_error);
  }

  uint64_t pending_job_count = 0;
  rc = qrmi_resource_status_pending_job_count(status, &pending_job_count);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    fprintf(stdout, "pending_job_count=%llu\n",
            (unsigned long long)pending_job_count);
  } else if (rc == QRMI_RETURN_CODE_UNSUPPORTED_FUNCTION_ERROR) {
    fprintf(stdout,
            "this resource has no queue / does not report pending job count\n");
  } else {
    fprintf(stderr, "qrmi_resource_status_pending_job_count() failed: %s\n",
            qrmi_get_last_error());
  }

  qrmi_resource_status_free(status);
  qrmi_resource_free(qrmi);

  return EXIT_SUCCESS;

error:
  qrmi_resource_free(qrmi);
  return EXIT_FAILURE;
}
