/*
 * This code is part of Qiskit.
 *
 * (C) Copyright IBM 2025.
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
#include <unistd.h>

#include "qrmi.h"

extern void load_dotenv();
extern const char *read_file(const char *);

int main(int argc, char *argv[]) {

  if (argc != 3) {
    fprintf(stderr, "direct_access <primitive input file> <program id>\n");
    return 0;
  }

  load_dotenv();

  const char *backend_name = getenv("QRMI_RESOURCE_ID");
  if (backend_name == NULL) {
    fprintf(stderr, "QRMI_RESOURCE_ID is not set.\n");
    return -1;
  }

  IBMDirectAccess *qrmi = qrmi_ibmda_new();
  bool is_accessible = false;
  int rc = qrmi_ibmda_is_accessible(qrmi, backend_name, &is_accessible);
  if (rc == QRMI_SUCCESS) {
    if (is_accessible == false) {
      fprintf(stderr, "%s cannot be accessed.\n", backend_name);
      return -1;
    }
  } else {
    fprintf(stderr, "qrmi_ibmda_is_accessible() failed.\n");
    return -1;
  }

  const char *acquisition_token = qrmi_ibmda_acquire(qrmi, backend_name);
  fprintf(stdout, "acquisition_token = %s\n", acquisition_token);

  rc = qrmi_ibmda_release(qrmi, acquisition_token);
  fprintf(stdout, "qrmi_ibmda_release rc = %d\n", rc);
  qrmi_free_string((char *)acquisition_token);

  const char *target = qrmi_ibmda_target(qrmi, backend_name);
  fprintf(stdout, "target = %s\n", target);
  qrmi_free_string((char *)target);

  const char *input = read_file(argv[1]);
  const char *job_id = qrmi_ibmda_task_start(qrmi, argv[2], input);
  if (job_id == NULL) {
    fprintf(stderr, "failed to start a task.\n");
    free((void*)input);
    return -1;
  }
  fprintf(stdout, "Job ID: %s\n", job_id);
  free((void*)input);

  TaskStatus status;
  while (1) {
    rc = qrmi_ibmda_task_status(qrmi, job_id, &status);
    if (rc != QRMI_SUCCESS || status != RUNNING) {
      break;
    }
    sleep(1);
  }

  rc = qrmi_ibmda_task_status(qrmi, job_id, &status);
  if (rc == QRMI_SUCCESS && status == COMPLETED) {
    const char *result = qrmi_ibmda_task_result(qrmi, job_id);
    fprintf(stdout, "%s\n", result);
    qrmi_free_string((char *)result);
  }

  qrmi_ibmda_task_stop(qrmi, job_id);

  qrmi_free_string((char *)job_id);

  qrmi_ibmda_free(qrmi);

  return 0;
}
