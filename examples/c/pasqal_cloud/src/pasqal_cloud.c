/*
 * This code is part of Qiskit.
 *
 * (C) Copyright IBM, Pasqal 2025.
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
    fprintf(stderr, "pasqal_cloud <backend_name> <input file>\n");
    return 0;
  }

  load_dotenv();

  PasqalCloud *qrmi = qrmi_pasqc_new(argv[1]);
  bool is_accessible = false;
  int rc = qrmi_pasqc_is_accessible(qrmi, &is_accessible);
  if (rc == QRMI_SUCCESS) {
    if (is_accessible == false) {
      fprintf(stderr, "%s cannot be accessed.\n", argv[1]);
      // return -1; // Fresnel currently inaccessible
    }
  } else {
    fprintf(stderr, "qrmi_pasqc_is_accessible() failed.\n");
    return -1;
  }

  const char *acquisition_token = qrmi_pasqc_acquire(qrmi);
  fprintf(stdout, "acquisition_token = %s\n", acquisition_token);

  const char *target = qrmi_pasqc_target(qrmi);
  fprintf(stdout, "target = %s\n", target);
  qrmi_free_string((char *)target);

  const char *input = read_file(argv[2]);
  const int shots = 100;
  
  fprintf(stdout, "input = %s\n", input);
  const char *job_id = qrmi_pasqc_task_start(qrmi, input, shots);
  if (job_id == NULL) {
    fprintf(stderr, "failed to start a task.\n");
    free((void*)input);
    return -1;
  }
  fprintf(stdout, "Job ID: %s\n", job_id);
  free((void*)input);

  TaskStatus status;
  while (1) {
    rc = qrmi_pasqc_task_status(qrmi, job_id, &status);
    fprintf(stdout, "rc = %d, status = %d\n", rc, status);
    if (rc != QRMI_SUCCESS || (status != RUNNING && status != QUEUED)) {
      break;
    }
    sleep(1);
  }

  rc = qrmi_pasqc_task_status(qrmi, job_id, &status);
  if (rc == QRMI_SUCCESS && status == COMPLETED) {
    const char *result = qrmi_pasqc_task_result(qrmi, job_id);
    fprintf(stdout, "%s\n", result);
    qrmi_free_string((char *)result);
  }
  else if (status == FAILED) {
    fprintf(stderr, "Failed.\n");
  }
  else if (status == CANCELLED) {
    fprintf(stderr, "Cancelled.\n");
  }

  // qrmi_pasqc_task_stop(qrmi, job_id); // what should be behaviour here?

  qrmi_free_string((char *)job_id);

  rc = qrmi_pasqc_release(qrmi, acquisition_token);
  fprintf(stdout, "qrmi_pasqc_release rc = %d\n", rc);
  qrmi_free_string((char *)acquisition_token);

  qrmi_pasqc_free(qrmi);

  return 0;
}
