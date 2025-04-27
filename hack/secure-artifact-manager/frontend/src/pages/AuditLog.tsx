import { Typography, Box } from "@mui/material";

function AuditLog() {
  return (
    <Box>
      <Typography variant="h5" gutterBottom>
        Audit Log
      </Typography>
      <Typography>
        Track all user actions and artifact events here.
      </Typography>
    </Box>
  );
}

export default AuditLog;