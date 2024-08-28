import axios from 'axios';

export const signUp = async (data) => {
  try {
    const url = 'http://172.18.0.3:8050/sign_up';
    console.log('Request URL:', url);
    const response = await axios.post(url, data, { withCredentials: true });
    return response.data;
  } catch (error) {
    console.error('Sign Up Error:', error.response ? error.response.data : error.message);
    throw error;
  }
};

export const signIn = async (data) => {
  try {
    const url = 'http://172.18.0.3:8050/sign_in';
    console.log('Request URL:', url);
    const response = await axios.post(url, data, { withCredentials: true });
    return response.data;
  } catch (error) {
    console.error('Sign In Error:', error.response ? error.response.data : error.message);
    throw error;
  }
};
