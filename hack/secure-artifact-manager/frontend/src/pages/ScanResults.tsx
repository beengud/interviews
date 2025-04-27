import { Typography, Box } from "@mui/material";

function ScanResults() {
  return (
    <Box>
      <Typography variant="h5" gutterBottom>
        Scan Results
      </Typography>
      <Typography>
        View results of artifact vulnerability scans here.
      </Typography>
    </Box>
  );
}

export default ScanResults;