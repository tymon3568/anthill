import * as fs from 'fs';

async function globalTeardown() {
  console.log('\n🧹 E2E Test Global Teardown');

  // Clean up test state file
  try {
    if (fs.existsSync('.e2e-state.json')) {
      fs.unlinkSync('.e2e-state.json');
      console.log('   ✓ Test state cleaned up');
    }
  } catch (error) {
    console.warn('   ⚠ Could not clean up test state');
  }

  console.log('   ✓ Global teardown complete');
}

export default globalTeardown;
